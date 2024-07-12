# Heatmap Client

This code is responsible for the client side of the heatmap generation process. 

The heavy lifting of generating the actual heatmap occurs in src/canvas
    
    The GPU is leveraged to generate these heatmaps, a large portion of the code is getting wgpu to play nicley in wasm

The ingest folder requests data from the server located in the heatmap-service directory