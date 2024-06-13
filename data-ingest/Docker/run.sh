# Make parent dir working dir so COPY commands in Dockerfile work
cd ..

# Build the docker image
sudo docker build -t data-ingest -f ./Docker/Dockerfile .

# Start a container and run ingest.py
sudo docker run --name data-ingest data-ingest

# Copy the output back to the local machine
sudo docker cp data-ingest:/data_ingest/sat_data.geojson ../sat_data.geojson

# Delete the container
sudo docker container rm -f data-ingest

# Modify copied file permissions
sudo chown $UID ./sat_data.geojson