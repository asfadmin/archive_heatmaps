# Archive Heatmap
The goal of this project is to rewrite and consolidate the existing codebases for creating heatmaps of satellite data

## Compiling Locally
1. Create a `cred.env` file in the same directory as heatmap.py
2. `cred.env`  should contain login credentials to the PostgreSQL DB, ie.
   ```
   export DB_HOST=change_me
   export DB_USERNAME=change_me
   export DB_PASSWORD=change_me
   export DB_NAME=change_me
   ```
3. After creating `cred.env` enter the command `source cred.env` in the terminal
4. Now you can run `python3 heatmap.py` to create a heatmap

## Dependancies
- PyShp
- matplotlib

## Contributing
Elliott Lewandowski
