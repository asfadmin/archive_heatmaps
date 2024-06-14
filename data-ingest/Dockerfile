
FROM ubuntu:latest

#Silent Install
ARG DEBIAN_FRONTEND=noninteractive

# Update / Install Stuff:
RUN apt-get update && apt-get upgrade -y
    #Non-Python Stuff:
RUN apt-get install -y libpq-dev postgresql postgis
    #Python Stuff:
RUN apt-get install -y python3-dev python3-pip python3-matplotlib python3-pyshp

#Setting up dummy user
RUN adduser heat
RUN su heat && mkdir ~/heatmap_proj

#Copy over project files
COPY ./heatmap.py home/heat/heatmap_proj/heatmap.py
COPY ./create_sql.py home/heat/heatmap_proj/create_sql.py
COPY ./antimeridian.py home/heat/heatmap_proj/antimeridian.py
COPY ./Resources home/heat/heatmap_proj/Resources
COPY ./cred.env home/heat/heatmap_proj/cred.env


