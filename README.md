# Archive Heatmap
The goal of this project is to rewrite and consolidate the existing codebases for creating heatmaps of satellite data to create an interactive heatmap

## Compiling Locally
1. Create a file named `.env` in the same directory as ingest.py
2. `.env`  should contain login credentials to the PostgreSQL DB, ie.
   ```
   export DB_HOST=change_me
   export DB_USERNAME=change_me
   export DB_PASSWORD=change_me
   export DB_NAME=change_me
   ```
3.  1)  If you have the dependencies installed locally you can now run `python3 ingest.py` and `sat_data.geojson` will be generated

    2)  If you have conda installed then you can create a conda enviornment using `env.yml` inside the `Docker` directory, you can then run `python3 ingest.py` inside this environment to generate `sat_data.geojson`
    
    3)  If you have docker installed then you can `cd` into `Docker` and then enter `./run.sh` which will generate `sat_data.geojson` 


## Dependancies
- postgis
- shapely
- pandas
- geopandas
- matplotlib

## Contributing
Elliott Lewandowski

Lily Larson
