
def generate_command(start, end):
    
    cmd = """SELECT g.granule_name, ST_AsText(shape), g.*
        
                FROM granule g 
                
                where g.platform_type in ('SA', 'SB') and
                g.data_granule_type in ('SENTINEL_1A_FRAME', 'SENTINEL_1B_FRAME'  ) and

                substr(granule_name, 8, 3) = 'GRD' and

                shape is not null and

                start_time between '""" + start.strftime("%x") + """' and '""" + end.strftime("%x") + """'    
            
                order   by start_time asc;"""
    
    return cmd