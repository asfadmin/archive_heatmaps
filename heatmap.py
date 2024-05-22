import psycopg2
import re
from connect_db import connect_to_db
import matplotlib.pyplot as plt
from numpy import random


def generate_heatmap():
    
    # Get a connection to the database
    conn = connect_to_db()
   
    #Create a cursor to execute an SQL command
    with conn.cursor() as curs:
        
        #SQL command to execute, currently hardcoded, should make this a passed parameter in later builds        
        SQL = """
                SELECT g.granule_name, ST_AsText(ST_Centroid(shape)), g.*
        
                FROM granule g 
                
                where g.platform_type in ('SA', 'SB') and
                g.data_granule_type in ('SENTINEL_1A_FRAME', 'SENTINEL_1B_FRAME'  ) and
\
                substr(granule_name, 8, 3) = 'GRD' and
\
                shape is not null and
\
                start_time between '2021-01-01' and '2021-02-01'    
            
                order   by start_time asc;"""
        
        #Execute SQL and store the results into data
        curs.execute(SQL)
        data = curs.fetchall()
        
        #Seperate gathered data into geographic categories
        cent = parse_center(data)
        f = open("parsed_output.txt","w")
        f.write(str(cent))
        f.close()
        
        location = break_data_on_geo(cent)
        f = open("loc_ouput.txt","w")
        f.write(str(location))
        f.close()
        
        
        ###########
        #  DEBUG  #
        ###########
        
        f = open("output.txt", "w")
        for lat in location.keys():
            for lon in location[lat].keys():
                f.write("Lat: " + str(lat) + "  Lon: " + str(lon) + "    Value: " + str(location[lat][lon]) + "\n")
        f.close()
        
        #Plot the resulting data pairs
        x = []
        y = []
        w = []
        
        #Sort the contents of location into x, y positions and corresponding weights
        for lat in location.keys():
            for lon in location[lat].keys():
                x.append(lon)
                y.append(lat)
                w.append(location[lat][lon])
        
        #Create a 2D Histogram of the data and export it as png
        plt.hist2d(x,y,[180,360],weights=w)
        plt.axis('off')
        plt.savefig("heatmap.png",bbox_inches='tight', pad_inches=0)        
        return
            
#Create a 2-dimensional dictionary, outer dict is latitude, inner dict is longitude,
#value is the number of samples data contains within the latitude and longitude pair
#
#Takes a list of tuples with latitude, longitude pairs       
def break_data_on_geo(data):

    loc = { i : {j : 0 for j in range(-180,181)} for i in range(-90,91) }
    close_latitude = 0
    close_longitude = 0
    
    for ele in data:
        #Convert current elements lat,lon cords to floats
        curr_lat = float(ele[1])
        curr_lon = float(ele[0])
        #print("Lat: " + str(curr_lat) + "Long: " + str(curr_lon))
        
          #Finds closest latitude
        for latitude in loc.keys():
            if abs(latitude - curr_lat) < abs(close_latitude - curr_lat):
                close_latitude = latitude
                
        #Finds closest longitude
        for longitude in loc[close_latitude].keys():
            if abs(longitude - curr_lon) < abs(close_longitude - curr_lon):
                close_longitude = longitude
        
       
        loc[close_latitude][close_longitude] += 1

    return loc

#Parses the center lat,lon of the passed data, returns a list of tuples
#Returns a list of long,lat pairs
def parse_center(data):
    center = []
    
    for ele in data:
        
        location = re.findall("-?\d+\.\d+", ele[1])
        
        center.append(location)
    
    f = open("loc_output.txt","w")
    for pair in center:
        f.write(str(pair) + "\n")
        if abs(float(pair[1])) > 90:
            print(str(pair))
    f.close()
    
    return center

generate_heatmap()