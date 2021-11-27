# Flatterer. Making JSON flatterer

## Introduction

An opinionated JSON to CSV/XLSX converter which tries to make a useful relational output for data analysis.

It aims to be fast and memory efficient.

### Install

```bash
pip install flatterer
```

Flatterer requires Python 3.6 or greater. It is written as a python extension in Rust but has binaries (wheels) for linux (x64 anylinux), macos (x64 and universal) and windows (x64, x86).  On other platforms a rust toolchain will need to be installed.

### Example JSON

Say you have a JSON data like this named `games.json`:

```json
[
  {
    "id": 1,
    "title": "A Game",
    "releaseDate": "2015-01-01",
    "platforms": [
      {"name":"Xbox"},
      {"name":"Playstation"}
    ],
    "rating": {
      "code": "E",
      "name": "Everyone"
    }
  },
  {
    "id": 2,
    "title": "B Game",
    "releaseDate": "2016-01-01",
    "platforms": [
      {"name":"PC"}
    ],
    "rating": {
      "code": "E",
      "name": "Everyone"
    }
  }
]
```


### Running Flatterer

Run the above file with flatterer.

```bash
flatterer games.json games_dir
```

See [](./options.md#option-reference) for details of additional command line options.


### Output Files


By running the above you will get the following files:

```bash
tree games_dir

games_dir/
├── csv
│   ├── games.csv
│   └── platforms.csv
├── data_package.json
├── fields.csv
└── ...
```

#### Main Table

`games.csv` contains:

|_link|_link_games|id |rating_code|rating_name|releaseDate|title |
|-----|---------- |---|-----------|-----------|-----------|------|
|1    |1          |1  |E          |Everyone   |2015-01-01 |A Game|
|2    |2          |2  |E          |Everyone   |2016-01-01 |B Game|


Special columns `_link` and `_link_games` are generated. `_link` is the primary key there unique per game. 

Also the `rating` sub-object is promoted to this table it has a one-to-one relationship with `games`. 
Sub-object properties are separated by '_'.  

#### One To Many Table

`platforms` is an array so is a one-to-many with games therefore needs its own table: 

`platforms.csv` contains:

|_link|_link_games|name|
|-----|----------|----|
|1.platforms.0|1 |Xbox|
|1.platforms.1|1 |Playstation|
|2.platforms.0|2 |PC  |

#### Link Fields

`_link` is the primary key for the `platforms` table too.  Every table contains a `_link_games` field to easily join to the main `games` table.

If there was a sub-array of `platforms` then that would have `_link`,  `_link_games` and  `_link_platforms` fields. 

To generalize this the `_link__<table_name>` fields joins to the `_link` field of `<table_name>` i.e the `_link__<table_name>` are the foreign keys refrencing `<table_name>._link`.

#### Fields CSV

`fields.csv` contains some metadata about the output tables:

|table_name |field_name|field_type|count|field_title|
|-----------|----------|----------|-----|----------|
|platforms  |_link     |text      |3    |_link     |
|platforms  |_link_games|text     |3    |_link_games|
|platforms  |name      |text      |3    |name      |
|games      |_link     |text     |2    | _link     |
|games      |_link_games|text    |2    | _link_games|
|games      |id        |number   |2    | id        |
|games      |rating_code|text    |2    | rating_code|
|games      |rating_name|text    |2    | rating_name|
|games      |releaseDate|date    |2    | releaseDate|
|games      |title     |text     |2    | title     |

The `field_type` column contains a type guess useful for inserting into a database. The `field_title` is the column heading in the CSV file or XLSX sheet, which is initally the same as the field_name.
After editing this file then you can rerun the transform:

```bash
flatterer games.json new_games_dir -f myfields.csv --only-fields
```

This can be useful for renameing columns, rearranging the field order or if you want to remove some fields the `--only-fields` flag will only include the fields in the edited file.

`data_package.json` contains metadata in the [Tabular Datapackge Spec](https://specs.frictionlessdata.io/tabular-data-package/#language)

[More information on the output formats.](./outputs.md#outputs)

```{toctree}
:hidden:
options
outputs
changelog
development
```
