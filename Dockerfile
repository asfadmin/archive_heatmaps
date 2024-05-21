
FROM ubuntu:latest

#Silent Install
ARG DEBIAN_FRONTEND=noninteractive

# Update / Install Stuff:
RUN apt-get update && apt-get upgrade -y
    #Non-Python Stuff:
RUN apt-get install -y libpq-dev postgresql
    #Python Stuff:
RUN apt-get install -y python3-dev python3-pip python3-matplotlib
    #Psycopg2 Stuff:
RUN apt install python3-psycopg2

