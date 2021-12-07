# Outputs

Both CSV and XLSX data can be produced with the following.

```bash
flatterer games.json games_dir --xlsx --csv
```

This will create the directory structure:

```bash
games_dir/
├── csv
│   ├── games.csv
│   └── platforms.csv
├── data_package.json
├── fields.csv
├── output.xlsx
├── postgresql
│   ├── postgresql_load.sql
│   └── postgresql_schema.sql
└── sqlite
    ├── sqlite_load.sql
    └── sqlite_schema.sql
```

## CSV

The `csv` directory contains a CSV file representing a relational table.  A new CSV file is produced for each one-to-many relationship found in the input JSON.

## XLSX

The XLSX output can be found in the output.xlsx file. It contains the same data as the CSV files with a tab per table.

## fields.csv

`fields.csv` contains some metadata about the output tables:

|table_name|field_name|field_type|count|
|----------|----------|----------|-----|
|platforms |_link     |text      |3    |
|platforms |_link_games|text     |3    |
|platforms |name      |text      |3    |
|games      |_link     |text     |2    |
|games      |_link_games|text    |2    |
|games      |id        |number   |2    |
|games      |rating_code|text    |2    |
|games      |rating_name|text    |2    |
|games      |releaseDate|date    |2    |
|games      |title     |text     |2    |

* `table_name`: The name of the table, which will be the same as the CSV file name without the `.csv` extension, or the XLSX sheet name.
* `field_name`: The name of field in the table.  This will be the same as the heading line in the CSV file and XLSX sheet.
* `field_type`: Type guess of the type of data within the field.  
* `count`: Amount of times that field appears int the JSON.

## Postgresql Files

When using a CSV output a `postgresql` directory is made which contains SQL to help load the CSV files into the database.  

### postgresql_schema.sql

Contains SQL with the basic schema definitions for the tables created.  The file looks like:

(postgres_schema)=
```sql
CREATE TABLE "games"(
    "_link" TEXT,
    "_link_games" TEXT,
    "id" NUMERIC,
    "title" TEXT,
    "releasedate" TIMESTAMP,
    "rating_code" TEXT,
    "rating_name" TEXT);

CREATE TABLE "platforms"(
    "_link" TEXT,
    "_link_games" TEXT,
    "name" TEXT);
```

Using the `psql` command line tool the schema can be loaded with something like:

```bash
psql postgresql://user:password@host/database -f games_dir/postgresql/postgresql_schema.sql
```

### postgresql_load.sql

This script imports the CSV files into the tables created by `postgresql_schema.csv`. This script requires the `psql` command line tool as it uses its `\copy` command but can easily be adapted to use the plain COPY command by removing the `\`.  The advantage of using `\copy` is that it does not have to be run on the server where the database is. It looks like:

```sql
\copy "games" from 'csv/games.csv' with CSV HEADER
\copy "platforms" from 'csv/platforms.csv' with CSV HEADER
```

and can be run by:

```bash
psql postgresql://user:password@host/database -f games_dir_/postgresql/postgresql_load.sql
```

## Sqlite Files.

When using a CSV output a `sqlite` directory is made which contains SQL to help load the CSV files into a sqlite database.  

### sqlite_schema.sql

Contains SQL with the basic schema definitions for the tables created. The file is the same as the [postgresql schema file](postgres_schema).

Using the `sqlite3` command line tool the schema can be loaded with something like:

```bash
cat games_dir/sqlite/sqlite_schema.sql | sqlite3 my_database.db
```

### sqlite_load.sql

This script imports the CSV files into the tables created by `sqlite_schema.csv`. This script requires the `sqlite3` command line tool as it uses its `.import` command. The contents looks like.

```sql
.mode csv 
.import 'csv/games.csv' games --skip 1 
.import 'csv/platforms.csv' platforms --skip 1 
```

and can be run by:

```bash
cat games_dir/sqlite/sqlite_load.sql | sqlite3 my_database.db
```

## data_package.json

Contains metadata in the [Tabular Datapackge Spec](https://specs.frictionlessdata.io/tabular-data-package/#language). I looks like:

```json
{
  "profile": "tabular-data-package",
  "resources": [
    {
      "profile": "tabular-data-resource",
      "name": "games",
      "schema": {
        "fields": [
          {
            "name": "_link",
            "type": "text",
            "count": 2
          },
          {
            "name": "_link_games",
            "type": "text",
            "count": 2
          },
          {
            "name": "id",
            "type": "number",
            "count": 2
          },
          {
            "name": "title",
            "type": "text",
            "count": 2
          },
          {
            "name": "releaseDate",
            "type": "date",
            "count": 2
          },
          {
            "name": "rating_code",
            "type": "text",
            "count": 2
          },
          {
            "name": "rating_name",
            "type": "text",
            "count": 2
          }
        ],
        "primaryKey": "_link"
      },
      "path": "csv/games.csv"
    },
    {
      "profile": "tabular-data-resource",
      "name": "platforms",
      "schema": {
        "fields": [
          {
            "name": "_link",
            "type": "text",
            "count": 3
          },
          {
            "name": "_link_games",
            "type": "text",
            "count": 3
          },
          {
            "name": "name",
            "type": "text",
            "count": 3
          }
        ],
        "primaryKey": "_link"
      },
      "path": "csv/platforms.csv"
    }
  ]
}
```


