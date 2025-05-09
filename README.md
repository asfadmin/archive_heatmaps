# Archive Heatmap
The goal of this project is to rewrite and consolidate the existing codebases for creating heatmaps of satellite data to create an interactive heatmap. This heatmap can currently display Sentinel-1 data with the ability to select date ranges and data types. The heatmap can also export to a png with formatting matching the old heatmap generation codebases.

## Generating a PNG
### Generate sat_data.geojson
1. Navigate to the `data-ingest` directory
2. Create a file named `.env` 
3. `.env`  should contain login credentials to the PostgreSQL DB, ie.
   ```
   export DB_HOST=change_me
   export DB_USERNAME=change_me
   export DB_PASSWORD=change_me
   export DB_NAME=change_me
   ```
4.  1)  If you have the dependencies installed locally you can now run `python3 ingest.py` and `sat_data.geojson` will be generated

    2)  If you have conda installed then you can create a conda enviornment using `env.yml` located inside the `Docker` directory, you can then run `python3 ingest.py` inside this environment to generate `sat_data.geojson`

### Setting up rust
1. Install rust, rust-lang.org has instructions on how to go about this
2. This project uses nightly features of rust, this means you will need a nightly version of rust, run `rustup toolchain install nightly-2025-04-01`
3. To swtich to a nightly build of rust run `rustup override set nightly-2025-04-01`

### Running the Service in Docker
1. Move `sat_data.geojson` to the `heatmap-service` directory, don't change the file name or the server will fail to find the data
2. Run the bash script `run-service-docker.sh` from the repository root, this will build a docker image to run the service and then start a container running the service.

The service has a startup time, if you attempt to access the client before the service is running the client will be locked into a loading screen until you reload. You can check if the service is ready by checking the processes CPU usage, it should be close to 0%. The service should log `Service Running!` in the console when it is close to finsihed with setup.

### Using the client
1. A release build of the client is available [here](https://asfadmin.github.io/archive_heatmaps/)
2. Once you are on the page select the date range, product type, and platform you would like to generate a heatmap for. Note that while the page may allow you to select ranges outside of those you generated `sat_data.geojson` with any data from those ranges will not be included. 
3. After the new selection of data has loaded click `Export to PNG` and wait for the file to download.


## Running the Service Locally
If you would prefer to run the service locally you must have a `.env` file in the `heatmap-service` directory, it should have the following fields:
```
SERVER_ADDRESS=127.0.0.1:8080 # the bind address for the microservice
CACHE_TTL=3600 # how long (in seconds) the cache should last for
GEO_JSON_PATH=/path/to/geojson
```
1. Move `sat_data.geojson` to the location specified in the `GEO_JSON_PATH` variable from your `.env` file
2. Navigate into the `heatmap-service` directory
3. Run `cargo run --release --package heatmap-service` from repository root 


## Contributing
Elliott Lewandowski<br>
Lily Larson
