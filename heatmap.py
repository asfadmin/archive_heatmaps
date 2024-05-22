import psycopg2
from connect_db import connect_to_db
import matplotlib.pyplot as plt
from numpy import random

def generate_heatmap():
    
    # Get a connection to the database
    conn = connect_to_db()
   
    #Create a cursor to execute an SQL command
    with conn.cursor() as curs:
        
        #SQL command to execute, currently hardcoded, should make this a passed parameter in later builds
        SQL = """SELECT g.granule_name, ST_AsText(shape), g.*
        
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
        location = break_data_on_geo(data)
        
        #Plot the resulting data pairs
        x = []
        y = []
        w = []
        
        #Sort the contents of location into x, y positions and corresponding weights
        for lon in location.keys():
            for lat in location[lon].keys():
                x.append(lon)
                y.append(lat)
                w.append(location[lon][lat])
        
        #Create a 2D Histogram of the data and export it as png
        plt.hist2d(x,y,[360,180],weights=w)
        plt.axis('off')
        plt.savefig("heatmap.png",bbox_inches='tight', pad_inches=0)
        
        return
            
#Create a 2-dimensional dictionary, outer dict is longitude, inner dict is latitude,
#value is the number of samples data contains within the longitude and latitude pair       
def break_data_on_geo(data):

    loc = { i : {j : 0 for j in range(-90,91)} for i in range(-180,181) }
    close_longitude = 0
    close_latitude = 0
    
    for ele in data:
        #Finds closest longitude
        for longitude in loc:
            if abs(longitude - ele[12]) < abs(close_longitude - ele[12]):
                close_longitude = longitude
                
        #Finds closest latitude
        for latitude in loc[close_longitude]:
            if abs(latitude - ele[11]) < abs(close_latitude - ele[11]):
                close_latitude = latitude
                
        loc[close_longitude][close_latitude] += 1

    return loc

generate_heatmap()