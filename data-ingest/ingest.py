from create_sql import generate_command
import geopandas as gps
import shapely
import os
import antimeridian
import quad_tree


# Generates heatmap.png based on the data
#   pulled in the generated SQL command
def ingest_data():

    ####################
    # Get Data From DB #
    ####################

    # Gather variables needed to generate command
    #   to dump the requested shapefile
    SQL = generate_command(data_type="'GRD'")
    db_host = os.getenv("DB_HOST")
    db_name = os.getenv("DB_NAME")
    db_username = os.getenv("DB_USERNAME")
    db_password = os.getenv("DB_PASSWORD")

    # Formulate and execute command to dump a shapefile
    cmd = (
        "pgsql2shp -f ./Resources/sat_data -h "
        + db_host
        + " -u "
        + db_username
        + " -P "
        + db_password
        + " "
        + db_name
        + ' "'
        + SQL
        + '"'
    )
    os.system(cmd)

    #####################################################
    # Read the dumped shapefile and format its contents #
    #####################################################

    # Read the data from the shapefile into a GeoDataFrame
    data_gdf = gps.read_file("./Resources/sat_data.shp")

    # Break polygons that cross the antimeridian
    i = 0
    while i < len(data_gdf["geometry"]):

        if antimeridian.check_antimeridian(
            list(data_gdf["geometry"][i].exterior.coords)
        ):

            # Store a reference to the original polygon and
            #   split on the antimeridian
            old_poly = data_gdf.iloc[i]
            split_polys = antimeridian.split_polygon(
                list(data_gdf["geometry"][i].exterior.coords)
            )

            # Remove the old polygon from the data frame,
            #   this increments our position in the dataframe
            data_gdf.drop(i, axis=0, inplace=True)

            # Add two new rows to the data frame using the split polygons
            temp = old_poly.to_dict()

            temp["geometry"] = shapely.Polygon(split_polys[0])
            data_gdf = data_gdf._append(temp, ignore_index=True)

            temp["geometry"] = shapely.Polygon(split_polys[1])
            data_gdf = data_gdf._append(temp, ignore_index=True)

        else:
            i += 1

    ###########################################
    # Merge similar records using a quad tree #
    ###########################################

    children = []
    for row in data_gdf.iterrows():
        curr = row[1].to_dict()
        curr["ancestors"] = []
        children.append(quad_tree.ChildNode(curr))

    # Create a quad tree with all of the satellite data
    #   and group similar satellite images
    tree = quad_tree.QuadTree([-180, 90], 360, 180, children)
    print("Original Children: " + str(len(tree.children)))

    tree.split(1)
    print("Split Children: " + str(tree.count_children()))

    #########################################################
    # Export the results of the quad tree to a geojson file #
    #########################################################

    # Convert quad tree data to a geojson string
    out_gdf = tree.to_gdf(data_gdf.crs)
    output = out_gdf.to_json()

    # Write the geojson string to a file
    file = open("sat_data.geojson", "w")
    file.write(output)
    file.close()

    # Clean up the resources folder
    os.system("rm -f Resources/sat_data.*")

    return


ingest_data()
