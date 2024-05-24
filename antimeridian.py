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

def split_polygon(poly: list):
    
    west_poly = []
    east_poly = []
    
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
        
    
    x_w = [ i[0] for i in west_poly]
    y_w = [ i[1] for i in west_poly]
    
    x_e = [ i[0] for i in east_poly]
    y_e = [ i[1] for i in east_poly]
    
    plt.plot(x_w,y_w,color="red")
    plt.plot(x_e,y_e,color="red")

    
    
    return
