import shapefile
import matplotlib.pyplot as plt

#Loops over a polygon and checks if it crosses the antimeridian
def check_antimeridian(poly: list):
    
    for curr_vertex in poly:
        #Pop halves the work if the polygon does not cross the antimeridian
        poly.pop(0)
        
        for check_vertex in poly:
            
            if abs(curr_vertex[0] - check_vertex[0]) >= 300:
                return True
    
    return False

#Split a polygon into two polygons on the antimeridian
def split_polygon(poly: list):
    
    west_poly = []
    east_poly = []
    
    #################################################
    # Create two polygons on either side of the map #
    #################################################
    
    for vertex in list(poly):
        
        vertex_list = list(vertex)
        
        if vertex_list[0] > 0:
            east_poly.append(vertex)
            
            #Place vertex in corresponding position on the other side of the prime meridian in the west polygon
            vertex_list[0] = vertex_list[0] - 360
            west_poly.append(vertex_list)
            
        else:
            west_poly.append(vertex)
            
            #Place 
            vertex_list[0] = vertex[0] + 360
            east_poly.append(vertex_list)      
    
    
    ###########################################
    # Create new vertexes on the antimeridian #
    ###########################################
    
    i = 0
    while i < len(west_poly) - 1:
        
        if not (((west_poly[i][0] < -180) and (west_poly[i+1][0] < -180)) or ((west_poly[i][0] > -180) and (west_poly[i+1][0] > -180))):
            
            #m=(y1-y2)/(x1-x2)
            slope = (west_poly[i][1] - west_poly[i+1][1])/(west_poly[i][0] - west_poly[i+1][0])
            #Distance between our point and the antimeridian
            distance = (-180) - west_poly[i][0]
            #The vertical shift required to translate our vertex onto the antimeridian
            shift = slope * distance
            
            #Form and insert a new vertex
            new_vertex = [-180, west_poly[i][1] + shift]
            west_poly.insert(i+1, new_vertex)
            
            #Skip the entry we just inserted
            i += 2
            
        else:
            i += 1
            
            
    #############################################
    # Remove all vertexes past the antimeridian #
    #############################################

    pop_west = []
    
    #Find indexes of all entries to remove
    for i in range(len(west_poly)):
        if west_poly[i][0] < -180:
            pop_west.append(i)
    
    #Remove entries in reverse order
    pop_west.sort(reverse=True)
    for i in pop_west:
        west_poly.pop(i)
    
    #Ensure that the polygon is still closed
    if west_poly[0] != west_poly[-1]:
        west_poly.append(west_poly[0])
            

        
    x_w = [ i[0] for i in west_poly]
    y_w = [ i[1] for i in west_poly]
    
    x_e = [ i[0] for i in east_poly]
    y_e = [ i[1] for i in east_poly]
    
    plt.plot(x_w,y_w)
    #plt.plot(x_e,y_e,color="red")
    
    return


