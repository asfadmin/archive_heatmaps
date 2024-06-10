from create_sql import generate_command
import geopandas as gpd
import pandas as pd
import shapely
import os
import antimeridian
import quad_tree
import time
import datetime


# Generates heatmap.png based on the data
#   pulled in the generated SQL command
def ingest_data():

    ####################
    # Get Data From DB #
    ####################

    t1 = time.time()

    # Gather variables needed to generate command
    #   to dump the requested shapefile
    SQL = generate_command(data_type="'OCN'", end=datetime.datetime(2021, 2, 1))
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

    t2 = time.time()
    print("Shapefile Dump: " + str(t2 - t1))

    #####################################################
    # Read the dumped shapefile and format its contents #
    #####################################################

    # Read the data from the shapefile into a GeoDataFrame
    data_gdf = gpd.read_file("./Resources/sat_data.shp")
    print(data_gdf)
    print(data_gdf["geometry"])

    t3 = time.time()
    print("Read Shapefile: " + str(t3 - t2))

    # Make a dict to store split polygons data
    split_dict = {}
    for key in data_gdf.columns:
        split_dict[key] = []

    sum = 0

    # Break polygons that cross the antimeridian
    i = 0
    while i < len(data_gdf["geometry"]):

        s = time.time()

        if antimeridian.check_antimeridian(
            list(data_gdf.iloc[i]["geometry"].exterior.coords)
        ):

            # Store a reference to the original polygon and
            #   split on the antimeridian
            old_poly = data_gdf.iloc[i]

            split_polys = antimeridian.split_polygon(
                list(data_gdf.iloc[i]["geometry"].exterior.coords)
            )

            # Remove the old polygon from the data frame,
            #   this increments our position in the dataframe
            data_gdf.drop(i, axis=0, inplace=True)

            # Reset index of the data frame
            data_gdf.reset_index(inplace=True, drop=True)

            # Copy the original polys data into two dicts
            west = old_poly.to_dict()
            east = old_poly.to_dict()

            # Update the dicts geometrys based on split polygons
            west["geometry"] = shapely.Polygon(split_polys[0])
            east["geometry"] = shapely.Polygon(split_polys[1])

            # Add data from two prior dicts to split_dict
            for key in split_dict.keys():
                split_dict[key].append(west[key])
                split_dict[key].append(east[key])

            sum += time.time() - s

        else:
            i += 1

    print("In while loop: " + str(sum))
    s = time.time()

    # Form a GeoDataFrame from the split polygons
    split_gdf = gpd.GeoDataFrame(split_dict, crs=data_gdf.crs)

    # Concatonate the two GeoDataFrames
    data_gdf = pd.concat([data_gdf, split_gdf])
    data_gdf.reset_index(inplace=True, drop=True)

    t4 = time.time()
    print("Out of while loop: " + str(t4 - s))
    print("Split on anti: " + str(t4 - t3))

    ###########################################
    # Merge similar records using a quad tree #
    ###########################################

    # Create a quad tree with all of the satellite data
    #   and group similar satellite images
    tree = quad_tree.QuadTree(data_gdf)
    tree.merge(0.1)

    t5 = time.time()
    print("Split Quad Tree: " + str(t5 - t4))

    #########################################################
    # Export the results of the quad tree to a geojson file #
    #########################################################

    # Convert quad tree data to a geojson string
    out_dict = tree.to_dict()
    out_gdf = gpd.GeoDataFrame(out_dict, crs=data_gdf.crs)
    output = out_gdf.to_json()

    t6 = time.time()
    print("To GeoJSON: " + str(t6 - t5))

    # Write the geojson string to a file
    file = open("sat_data.geojson", "w")
    file.write(output)
    file.close()

    t7 = time.time()
    print("Write to file: " + str(t7 - t6))
    print("Total Time: " + str(t7 - t1))

    # Clean up the resources folder
    os.system("rm -f Resources/sat_data.*")

    return


ingest_data()
