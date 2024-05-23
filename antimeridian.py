import shapefile

#Loops over a polygon and checks if it crosses the antimeridian
#
def check_antimeridian(poly: list):
    
    for curr_coord in poly:
        
        #Pop halves the work if the polygon does not cross the antimeridian
        poly.pop(0)
        
        for check_coord in poly:
            
            if abs(curr_coord[0] - check_coord[0]) >= 300:
                return True
    
    return False