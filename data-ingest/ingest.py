from create_sql import generate_command
from dotenv import load_dotenv
import geopandas as gpd
import pandas as pd
import antimeridian
import data_merger
import datetime
import shapely
import os


# Generates sat_data.geojson based on the data
#   pulled in the generated SQL command
def ingest_data():

    ####################
    # Get Data From DB #
    ####################

    # Generate SQL command to filter data in PostgreSQL DB
    SQL = generate_command(data_type="'OCN', 'SLC', 'GRD'")

    # Load credentials to connect to DB
    load_dotenv()
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
    os.system("mkdir Resources")
    os.system(cmd)

    #####################################################
    # Read the dumped shapefile and format its contents #
    #####################################################

    # Read the data from the shapefile into a GeoDataFrame
    data_gdf = gpd.read_file("./Resources/sat_data.shp")

    # Make a dict to store split polygons data
    split_dict = {}
    same_dict = {}
    for key in data_gdf.columns:
        split_dict[key] = []
        same_dict[key] = []

    # Break polygons that cross the antimeridian
    i = 0
    for row in data_gdf.iterrows():

        # Store the data of the current row
        row_data = row[1]

        # Check if the polygon crosses the antimeridian
        if antimeridian.check_antimeridian(list(row_data["geometry"].exterior.coords)):

            # Split the polygon into two new polygons
            split_polys = antimeridian.split_polygon(
                list(row_data["geometry"].exterior.coords)
            )

            # Copy the original polys data into two dicts
            west = row_data.to_dict()
            east = row_data.to_dict()

            # Update the dicts geometrys based on split polygons
            west["geometry"] = shapely.Polygon(split_polys[0])
            east["geometry"] = shapely.Polygon(split_polys[1])

            # Add data from two prior dicts to split_dict
            for key in split_dict.keys():
                split_dict[key].append(west[key])
                split_dict[key].append(east[key])

        # Add the data as it was to the same_dict
        else:
            for key in same_dict.keys():
                same_dict[key].append(row_data[key])

        i += 1

    # Form a GeoDataFrame from the two dicts
    split_gdf = gpd.GeoDataFrame(split_dict, crs=data_gdf.crs)
    same_gdf = gpd.GeoDataFrame(same_dict, crs=data_gdf.crs)

    # Concatonate the two GeoDataFrames
    data_gdf = pd.concat([same_gdf, split_gdf])
    data_gdf.reset_index(inplace=True, drop=True)

    #########################
    # Merge similar records #
    #########################

    # Create a DataMerger with all of the satellite data
    #   and merge similar satellite images
    merger = data_merger.DataMerger(data_gdf)
    merger.merge(1.0)

    #########################################################
    # Export the results of the quad tree to a geojson file #
    #########################################################

    # Convert merged data to a geojson string
    out_dict = merger.to_dict()
    out_gdf = gpd.GeoDataFrame(out_dict, crs=data_gdf.crs)
    output = out_gdf.to_json()

    # Write the geojson string to a file
    file = open("sat_data.geojson", "w")
    file.write(output)
    file.close()

    # Clean up the resources folder
    os.system("rm -rf ./Resources")

    return


ingest_data()
