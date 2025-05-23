# Data Ingest

This directory handles pulling data from a PostgreSQL database and formatting. The file it generates is served to the heatmap-client from the heatmap-service

## Compiling Locally

1. Create a file named `.env` 
2. `.env`  should contain login credentials to the PostgreSQL DB, ie.
   ```
   export DB_HOST=change_me
   export DB_USERNAME=change_me
   export DB_PASSWORD=change_me
   export DB_NAME=change_me
   ```
3.  To select a date range and data type to generate data for go into `create_sql.py` and change the default values of `generate_command` as you desire 
4.  1)  If you have docker installed then you can cd into `./Docker` and run `./run.sh` to generate `sat_data.geojson`
    2)  If you have the dependencies installed locally you can now run `python3 ingest.py` and `sat_data.geojson` will be generated

    3)  If you have conda installed then you can create a conda enviornment using `env.yml` inside the `Docker` directory, you can then run `python3 ingest.py` inside this environment to generate `sat_data.geojson`


## Dependancies
- postgis
- shapely
- pandas
- geopandas
- dotenv