from create_sql import generate_command
import matplotlib.pyplot as plt
import datetime as date
import geopandas as gps
import pandas
import shapely
import os
import antimeridian
import quad_tree



#Generates heatmap.png based on the data pulled in the contained SQL command
#
def generate_heatmap():
    
    ####################
    # Get Data From DB #
    ####################
    
    #Gather variables needed to generate command to dump the requested shapefile  
    SQL = generate_command(data_type="'OCN'")
    db_host = os.getenv("DB_HOST")
    db_name = os.getenv("DB_NAME")
    db_username = os.getenv("DB_USERNAME")
    db_password = os.getenv("DB_PASSWORD")
    
    #Formulate and execute command to dump a shapefile
    cmd = "pgsql2shp -f ./Resources/sat_data -h " + db_host + " -u " + db_username + " -P " + db_password + " " + db_name + ' "' + SQL + '"'
    os.system(cmd)
    
    ##############################################
    # Plot the shapefiles in Resources directory #
    ##############################################
        
        ##############
        # World Data #
        ##############
    
    #Read the data from the shapefile into a GeoDataFrame  
    world_gdf = gps.read_file("./Resources/world-boundaries.shp")
    
    ax = world_gdf.plot(alpha=0.4,color="black")
    
        ############
        # Sat Data #
        ############
       
    #Read the data from the shapefile into a GeoDataFrame    
    data_gdf = gps.read_file("./Resources/sat_data.shp")       
    
    i = 0
    while i < len(data_gdf["geometry"]):
        
        if antimeridian.check_antimeridian(list(data_gdf["geometry"][i].exterior.coords)):
            
            #Store a reference to the original polygon and split on the antimeridian
            old_poly = data_gdf.iloc[i] ##FIX THIS, ILOC IS DEPRACATED
            split_polys = antimeridian.split_polygon(list(data_gdf["geometry"][i].exterior.coords))
           
            #Remove the old polygon from the data frame, this increments our position in the dataframe
            data_gdf.drop(i,axis=0,inplace=True)
            
            #Add two new rows to the end of the data frame using the split polygons
            temp = old_poly.to_dict()
            
            temp["geometry"] = shapely.Polygon(split_polys[0])
            data_gdf = data_gdf._append(temp, ignore_index = True)
            
            temp["geometry"] = shapely.Polygon(split_polys[1])
            data_gdf = data_gdf._append(temp, ignore_index = True)
            
        else:
            i += 1
            
    children = []
    for poly in data_gdf["geometry"]:
                children.append(quad_tree.ChildNode(poly, []))
            
    #Create a quad tree with all of the satellite data and group similar satellite images       
    tree = quad_tree.QuadTree([-180,90],360,180,children)
    print("Original Children: " + str(len(tree.children)))
    tree.split(1)
    print("Split Children: " + str(tree.count_children()))
    
    #Plot the quad tree which contains the satellit data
    tree.plot(ax=ax)
    
    tree.print()
    
    #Show the resulting plot    
    plt.show()
    
    #Clean up the resources folder
    os.system("rm -f Resources/sat_data.*")   
    
    return

generate_heatmap()
