import datetime as date

# Generates an SQL command based on the passed parameters or the defaults
#
# ARGS:
#
#   start               The start date for the heatmap
#   end                 The end date for the heatmap
#   platform_type       The platform to generate a heatmap for, ie SA or SB
#   data_type           The type of data to generate a heatmap for, ie. GRD, SLC, or OCN
#


def generate_command(
    start=date.datetime(2021, 1, 1),
    end=date.datetime(2021, 1, 10),
    platform_type="'SA', 'SB'",
    data_type="'OCN', 'SLC', 'GRD'",
):

    cmd = (
        """
        SELECT 
                g.granule_name, 
                g.platform_type, 
                g.data_sensor_type, 
                g.start_time, 
                g.shape
        FROM granule g
        WHERE 
                g.platform_type IN ("""
                + platform_type
                + """) AND
                g.data_granule_type IN ('SENTINEL_1A_FRAME', 'SENTINEL_1B_FRAME'  ) AND
                substr(g.granule_name, 8, 3) IN ("""
                + data_type
                + """) AND
                g.shape IS NOT null AND
                g.start_time BETWEEN '"""
                + start.strftime("%x")
                + """' and '"""
                + end.strftime("%x")
                + """'
        ORDER BY shape ASC"""
    )

    return cmd
