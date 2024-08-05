# Archive Heatmap
The goal of this project is to rewrite and consolidate the existing codebases for creating heatmaps of satellite data to create an interactive heatmap

## Compiling Locally
### Generate sat_data.geojson
1. Navigate to the data-ingest directory, `cd data-ingest`
2. Create a file named `.env` 
3. `.env`  should contain login credentials to the PostgreSQL DB, ie.
   ```
   export DB_HOST=change_me
   export DB_USERNAME=change_me
   export DB_PASSWORD=change_me
   export DB_NAME=change_me
   ```
4.  1)  If you have the dependencies installed locally you can now run `python3 ingest.py` and `sat_data.geojson` will be generated

    2)  If you have conda installed then you can create a conda enviornment using `env.yml` inside the `Docker` directory, you can then run `python3 ingest.py` inside this environment to generate `sat_data.geojson`

### Setting up rust
1. Install rust, rust-lang.org is the page you're looking for
2. This project uses nightly features of rust, this means you will need a nightly version of rust, run `rustup toolchain install nightly`
3. To swtich to a nightly build of rust run `rustup override set nightly`

### Setting up the server
1. Move `sat_data.geojson` to the `heatmap-service` directory, don't change the file name or the server will fail to find the data
2. Navigate into the `heatmap-service` directory, `cd heatmap-service`
3. Run `cargo run` in the terminal and you now have a locally running version of the server, if the terminal you entered this command into closes you will need to repeat this step in a new terminal

### Setting up the client
1. Navigate to the `heatmap-client` directory, `cd heatmap-client`
2. Install trunk, run `cargo binstall trunk`
3. Run `trunk serve --open`, this should open a page in your default browser, if you would prefer the command not open a page remove `--open` and it will serve the client without opening a new page

## Contributing
Elliott Lewandowski

Lily Larson
