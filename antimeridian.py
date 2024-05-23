import shapefile

def check_antimeridian(poly: list):
    
    for curr_coord in poly:
        poly.pop(0)
        for check_coord in poly:
            if abs(curr_coord[0] - check_coord[0]) >= 300:
                return True
        
    
    return False