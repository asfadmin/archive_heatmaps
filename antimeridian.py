import matplotlib.pyplot as plt


#Loops over a polygon and checks if it crosses the antimeridian
def check_antimeridian(poly: list):
    
    for curr_vertex in poly:
        #Pop here halves the work if the polygon does not cross the antimeridian
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
    
    for vertex in poly:
        
        if vertex[0] > 0:
            east_poly.append(vertex)
            
            #Place vertex in corresponding position on the other side of the prime meridian in the west polygon
            reflected = [vertex[0] - 360, vertex[1]]
            west_poly.append(reflected)
            
        else:
            west_poly.append(vertex)
            
            #Place 
            reflected = [vertex[0] + 360, vertex[1]]
            east_poly.append(reflected)      
    
    all_poly = [west_poly, east_poly]
    
    ###########################################
    # Create new vertexes on the antimeridian #
    ###########################################
    
    for dir_poly in all_poly:
        
        i = 0
        while i < len(dir_poly) - 1:
            
            if edge_crosses_antimeridian(dir_poly[i],dir_poly[i+1]):

                #m=(y1-y2)/(x1-x2)
                slope = (dir_poly[i][1] - dir_poly[i+1][1])/(dir_poly[i][0] - dir_poly[i+1][0])
                
                #Distance between our point and the antimeridian      
                distance = 180 - abs(dir_poly[i][0])
                
                #Accounts for the negative values being flipped by the abs()
                if dir_poly[i][0] < 0:
                    distance *= -1
                    
                #The vertical shift required to translate our vertex onto the antimeridian
                shift = slope * distance
                
                #Form and insert a new vertex
                if dir_poly[i][0] > 0:
                    new_vertex = [180, dir_poly[i][1] + shift]
                else:
                    new_vertex = [-180, dir_poly[i][1] + shift]
                
                dir_poly.insert(i+1, new_vertex)#DEBUG
                
                #Skip the entry we just inserted
                i += 2
                
            else:
                i += 1      
                
        #############################################
        # Remove all vertexes past the antimeridian #
        #############################################

        pop_dir = []
        
        #Find indexes of all entries to remove
        for i in range(len(dir_poly)):
            if abs(dir_poly[i][0]) > 180:
                pop_dir.append(i)
        
        #Remove entries in reverse order
        pop_dir.sort(reverse=True)
        for i in pop_dir:
            dir_poly.pop(i)
        
        #Ensure that the polygon is still closed
        if dir_poly[0] != dir_poly[-1]:
            dir_poly.append(dir_poly[0])
            
    for dir_poly in all_poly:
        x = [ i[0] for i in dir_poly]
        y = [ i[1] for i in dir_poly]
        plt.plot(x,y)
    
    return

#Checks if a line between two points would cross the antimeridian
def edge_crosses_antimeridian(vertex1, vertex2):
    
    p1 = abs(vertex1[0])
    p2 = abs(vertex2[0])
    
    #If the two points are not on the same said of the antimeridian return true
    if not (((p1 > 180) and (p2 > 180)) or ((p1 < 180) and (p2 < 180))):
        return True
    
    return False