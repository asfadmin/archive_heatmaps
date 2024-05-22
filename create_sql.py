import datetime

def generate_command(start, end, data_type, platform_type):
    
    cmd = """SELECT g.granule_name, ST_AsText(ST_Centroid(shape)), g.*
        
                FROM granule g 
                
                where g.platform_type in (""" + platform_type + """) and
                g.data_granule_type in ('SENTINEL_1A_FRAME', 'SENTINEL_1B_FRAME'  ) and

                substr(granule_name, 8, 3) = '""" + data_type + """' and

                shape is not null and

                start_time between '""" + start.strftime("%x") + """' and '""" + end.strftime("%x") + """'    
            
                order   by start_time asc;"""
    
    return cmd