import psycopg2
import re
from connect_db import connect_to_db
from create_sql import generate_command
import matplotlib.pyplot as plt
import datetime as date
from numpy import random, arange
import shapefile
import os


#Generates heatmap.png based on the data pulled in the contained SQL command
#
def generate_heatmap():
    
    ####################
    # Get Data From DB #
    ####################
    
    # Get a connection to the database
    conn = connect_to_db()
   
    #Create a cursor to execute an SQL command
    with conn.cursor() as curs:

        #SQL command to execute, currently hardcoded, should make this a passed parameter in later builds     
        SQL = generate_command(data_type="'GRD', 'SLC'")
        db_host = os.getenv("DB_HOST")
        db_name = os.getenv("DB_NAME")
        db_username = os.getenv("DB_USERNAME")
        db_password = os.getenv("DB_PASSWORD")
    
        cmd = "pgsql2shp -f shape_dump/sat_data -h " + db_host + " -u " + db_username + " -P " + db_password + " " + db_name + ' "' + SQL + '"'
        os.system(cmd)

        #Execute SQL and store the results into data
        curs.execute(SQL)
        data = curs.fetchall()
        
    ########################
    # Generate heatmap.png #
    ########################
        
        #Seperate gathered data into geographic categories
        cent = parse_center(data)
        location = break_data_on_geo(cent)

        #Sort the contents of location into x, y positions and corresponding weights
        x = []
        y = []
        w = []
        
        for lat in location.keys():
            for lon in location[lat].keys():
                x.append(lon)
                y.append(lat)
                w.append(location[lat][lon])
                
        plt.hist2d(x,y,[360,360],weights=w)
                
        sf = shapefile.Reader("./Resources/world-administrative-boundaries.shp")

        for shape in sf.shapeRecords():
            for i in range(len(shape.shape.parts)):
                i_start = shape.shape.parts[i]
                
                if i==len(shape.shape.parts)-1:
                    i_end = len(shape.shape.points)
                else:
                    i_end = shape.shape.parts[i+1]
                    
                x = [i[0] for i in shape.shape.points[i_start:i_end]]
                y = [i[1] for i in shape.shape.points[i_start:i_end]]
                
                plt.plot(x,y,color="black")
        
        #Create a 2D Histogram of the data and export it as png
        
        plt.axis('off')
        plt.savefig("heatmap.png",bbox_inches='tight', pad_inches=0)        
        
        return
 
#Categorizes data into distinct geographic regions           
#
#ARGS:
#
#   data            list of tuples containng (longitude, latitude)
#     
#RETURNS:
#
#   A 2D dictionary, outer dict is latitude, inner dict is longitude,
#       value is the number of samples at the corresponding coordinates
#
def break_data_on_geo(data):

    loc = { i : {j : 0 for j in range(-180, 181)} for i in arange(-90, 90, 0.5) }
    close_latitude = 0
    close_longitude = 0
    
    for ele in data:
        #Convert current elements lat,lon cords to floats
        curr_lat = float(ele[1])
        curr_lon = float(ele[0])
        
        #Finds closest latitude
        for latitude in loc.keys():
            if abs(latitude - curr_lat) < abs(close_latitude - curr_lat):
                close_latitude = latitude
                
        #Finds closest longitude
        for longitude in loc[close_latitude].keys():
            if abs(longitude - curr_lon) < abs(close_longitude - curr_lon):
                close_longitude = longitude
        
        #Increment the value at the correct lat,lon pair
        loc[close_latitude][close_longitude] += 1

    return loc

#Parses the center lat,lon of the passed data
#
#ARGS:
#
#   data            A list of lists, the inner lists second entry must be the center lat,long
#
#RETURNS:
#
#   A 2D list, the inner list is composed of strings corresponding to coordinates
#
def parse_center(data):

    center = []
    
    for ele in data:
        
        location = re.findall("-?\d+\.\d+", ele[1])
        center.append(location)
    
    return center




SQL = generate_command(data_type="'GRD', 'SLC'")
db_host = os.getenv("DB_HOST")
db_name = os.getenv("DB_NAME")
db_username = os.getenv("DB_USERNAME")
db_password = os.getenv("DB_PASSWORD")

