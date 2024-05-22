import psycopg2
from os import getenv


#Returns a connection to the PostgreSQL DB specified in the envioronment variables below:
#
#   Vars                    Purpose
#  DB_HOST            Stores the address of the DB host
#  DB_NAME            Stores the name of the DB
#  DB_USERNAME        Stores Username for logging into DB
#  DB_PASSWORD        Stores Password for the Username above
#
def connect_to_db():
    
    #Get the env variables representing the user credentials and database
    db_host = getenv("DB_HOST")
    db_name = getenv("DB_NAME")
    db_username = getenv("DB_USERNAME")
    db_password = getenv("DB_PASSWORD")
    
    #Connect to the database using the credentials specified in dev.env and return that connection
    try:
        conn = psycopg2.connect(
            user=db_username, 
            password=db_password, 
            host=db_host, 
            dbname=db_name)
        print("Connected to the PostgreSQL server.")
        return conn
    #Print an error if the connection fails
    except (psycopg2.DatabaseError, Exception) as error:
        print(error)