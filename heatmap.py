from create_sql import generate_command
import matplotlib.pyplot as plt
import datetime as date
import shapefile
import os


#Generates heatmap.png based on the data pulled in the contained SQL command
#
def generate_heatmap():
    
####################
# Get Data From DB #
####################
    
    #Gather variables needed to generate command to dump the requested shapefile  
    SQL = generate_command(data_type="'GRD', 'SLC'")
    db_host = os.getenv("DB_HOST")
    db_name = os.getenv("DB_NAME")
    db_username = os.getenv("DB_USERNAME")
    db_password = os.getenv("DB_PASSWORD")
    
    #Formulate and execute command to dump a shapefile
    cmd = "pgsql2shp -f Resources/sat_data -h " + db_host + " -u " + db_username + " -P " + db_password + " " + db_name + ' "' + SQL + '"'
    os.system(cmd)
    
########################
# Generate heatmap.png #
########################
    
    #Plot the data from the satellite into the image
    data_sf = shapefile.Reader("./Resources/sat_data.shp")        
            
    for shape in data_sf.shapeRecords():
         for i in range(len(shape.shape.parts)):
            i_start = shape.shape.parts[i]
            
            if i==len(shape.shape.parts)-1:
                i_end = len(shape.shape.points)
            else:
                i_end = shape.shape.parts[i+1]
                
            x = [i[0] for i in shape.shape.points[i_start:i_end]]
            y = [i[1] for i in shape.shape.points[i_start:i_end]]
            
            plt.plot(x,y,color="green")
            
    #Plot the world map into the image  
    world_sf = shapefile.Reader("./Resources/world-boundaries.shp")

    for shape in world_sf.shapeRecords():
        for i in range(len(shape.shape.parts)):
            i_start = shape.shape.parts[i]
            
            if i==len(shape.shape.parts)-1:
                i_end = len(shape.shape.points)
            else:
                i_end = shape.shape.parts[i+1]
                
            x = [i[0] for i in shape.shape.points[i_start:i_end]]
            y = [i[1] for i in shape.shape.points[i_start:i_end]]
            
            plt.plot(x,y,color="black")
    
    
    #Save the current plot to a .png file
    plt.axis('off')
    plt.savefig("heatmap.png",bbox_inches='tight', pad_inches=0)     
    
    #Clean up the resources folder
    os.system("rm -f Resources/sat_data.*")   
    
    return


generate_heatmap()

