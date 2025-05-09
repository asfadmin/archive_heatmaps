# Docker needs to be able to access heatmap-api
cd ..

# Build docker image to run heatmap-service
sudo docker build -t heatmap-service -f heatmap-service/Dockerfile .

# Start a container running the service
sudo docker run -p 8000:8000 heatmap-service
