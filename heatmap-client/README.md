# Heatmap Client

This code is responsible for the client side of the heatmap generation process. 

`./src/canvas` does the heavy lifting of generating the actual heatmap 
    
    The GPU is leveraged to generate these heatmaps, a large portion of the code is getting wgpu to play nicley in wasm

`./src/ingest` requests data from the server located in the heatmap-service directory

`./src/ui` This contains the user interface that is overlayed onto the heatmap

`./assets` contains static assets used in the client, ie. colormap textures and resources to export a png

## Compiling Locally
1. Ensure rust is on a nightly build, you can check with `rustup toolchain list`
2. Install trunk, run `cargo binstall trunk`
3. Run `trunk serve --open`, this should open a page in your default browser, if you would prefer the command not open a page remove `--open` and it will serve the client without opening a new page