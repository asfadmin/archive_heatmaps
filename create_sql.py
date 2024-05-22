import datetime as date

#Generates an SQL command based on the passed parameters or the defaults
#
#ARGS:
#
#   start               The start date for the heatmap
#   end                 The end date for the heatmap
#   platform_type       The platform to generate a heatmap for, ie SA or SB
#   data_type           The type of data to generate a heatmap for, ie. GRD or SLC
#

def generate_command(start=date.datetime(2021,1,1), 
                     end=date.datetime(2021,2,1), 
                     platform_type="'SA', 'SB'", 
                     data_type="GRD"):
    
    cmd = """SELECT g.granule_name, ST_AsText(ST_Centroid(shape)), g.*
        
                FROM granule g 
                
                where g.platform_type in (""" + platform_type + """) and
                g.data_granule_type in ('SENTINEL_1A_FRAME', 'SENTINEL_1B_FRAME'  ) and

                substr(granule_name, 8, 3) = '""" + data_type + """' and

                shape is not null and

                start_time between '""" + start.strftime("%x") + """' and '""" + end.strftime("%x") + """'    
            
                order   by start_time asc;"""
    
    return cmd