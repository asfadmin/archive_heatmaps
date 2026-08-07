
# Archive Heatmap
The goal of this project is to rewrite and consolidate the existing codebases for creating heatmaps of the Sentinel-1 data archive into an interactive heatmap. This heatmap currently has the ability to select date ranges, product types, and platform.
<img width="5120" height="2712" alt="cumulative" src="https://github.com/user-attachments/assets/9a783818-c554-4497-b28d-d382a29e1c1d" />


## Architecture Diagram
<img width="8709" height="5347" alt="Interactive-Heatmap-Architecture-Diagram" src="https://github.com/user-attachments/assets/fce662c7-6f12-4c30-ae51-9985e305f796" />

## Using the Client
1. A release build of the client is available at https://heatmaps-live.sp.asf.alaska.edu/
2. Once you are on the page select the date range, product type, and platform you would like to generate a heatmap for and press submit!

## Directory Contents
`./src/canvas` does the heavy lifting of generating the actual heatmap 
    
    The GPU is leveraged to generate these heatmaps, a large portion of the code is getting wgpu to play nicley in wasm

`./src/ingest` requests and meshes data from a public S3 bucket using [DuckDB](https://duckdb.org/)

`./src/ui` This contains the user interface that is overlayed onto the heatmap

`./assets` contains static assets used in the client, ie. colormap textures

## Running Locally
1. Install rust, [rust-lang.org](https://rust-lang.org) has instructions on how to go about this
2. This project uses nightly features of rust, this means you will need a nightly version of rust, run `rustup toolchain install nightly-2026-05-14`
3. To swtich to a nightly build of rust run `rustup override set nightly-2026-05-14`
4. Install the `wasm32-unknown-unknown` target with: `rustup target add wasm32-unknown-unknown`
5. Install trunk, instructions can be found [here](https://crates.io/crates/trunk) 
6. From the root of the project run `trunk serve --open`. 


## Attribution
World Continent data comes from hub.arcgis.com/datasets/esri::world-continents/explore
Sentinel-1 Data is provided by the Alaska Satellite Facility

## Contributers
Elliott Lewandowski<br>
Lily Larson
