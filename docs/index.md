# Flatterer. Making JSON flatterer

<img width="500" href="#" src="./_static/flatterer-with-text.svg">

## Introduction

An opinionated JSON to CSV/XLSX/SQLITE/PARQUET converter which tries to make a useful relational output for data analysis.

## Rationale

When receiving a JSON file where the structure is deeply nested or not well specified, it is hard to determine what the data contains. Also, even after knowing the JSON structure, it requires a lot of time to work out how to flatten the JSON into a relational structure to do data analysis on and to be part of a data pipeline. 

Flatterer aims to be the first tool to go to when faced with the above problem.  It may not be the tool that you end up using to flatten the JSON in your data pipeline, as hand written flattening may be required, but it could be.  It has many benefits over most hand written approaches:

* It is fast, written in rust but with python bindings for ease of use.  It can be 10x faster than hand written python flattening.
* Memory efficient.  Uses a custom streaming JSON parser to mean that long list of objects nested with the JSON will be streamed, so not much data needs to be loaded into memory at once.
* Fast memory efficient output to CSV/XLSX/SQLITE/PARQUET
* Uses best practice that has been learnt from flattening JSON countless times, such as generating keys to link one-to-many tables to their parents.


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

#### CLI

Run the above file with flatterer.

```bash
flatterer games.json games_dir
```

See [](./options.md#option-reference) for details of additional command line options.

#### As python library

```python
import flatterer
output = flatterer.flatten('games.json', 'games_dir')
```

See [](./library.md#python-library) for more details.

### Output Files

By running the above you will get the following files:

```bash
tree games_dir

games_dir/
├── csv
│   ├── games.csv
│   └── platforms.csv
├── datapackage.json
├── fields.csv
└── ...
```

#### Main Table

`games.csv` contains:

|_link|id |rating_code|rating_name|releaseDate|title |
|----|---|-----------|-----------|-----------|------|
|0   |1  |E          |Everyone   |2015-01-01 |A Game|
|1   |2  |E          |Everyone   |2016-01-01 |B Game|


Special column `_link` is generated. `_link` is the primary key there unique per game. 

Also the `rating` sub-object is promoted to this table it has a one-to-one relationship with `games`. 
Sub-object properties are separated by '_'.  

#### One To Many Table

`platforms` is an array so is a one-to-many with games therefore needs its own table: 

`platforms.csv` contains:

|_link|_link_games|name|
|-----|----------|----|
|0.platforms.0|0 |Xbox|
|0.platforms.1|0 |Playstation|
|1.platforms.0|1 |PC  |

#### Link Fields

`_link` is the primary key for the `platforms` table too.  Every table except `games` table, contains a `_link_games` field to easily join to the main `games` table.

If there was a sub-array of `platforms` then that would have `_link`,  `_link_games` and  `_link_platforms` fields. 

To generalize this the `_link__<table_name>` fields joins to the `_link` field of `<table_name>` i.e the `_link__<table_name>` are the foreign keys referencing `<table_name>._link`.

#### Fields CSV

`fields.csv` contains some metadata about the output tables:

|table_name |field_name|field_type|count|field_title|
|-----------|----------|----------|-----|----------|
|platforms  |_link     |text      |3    |_link     |
|platforms  |_link_games|text     |3    |_link_games|
|platforms  |name      |text      |3    |name      |
|games      |_link     |text     |2    | _link     |
|games      |id        |number   |2    | id        |
|games      |rating_code|text    |2    | rating_code|
|games      |rating_name|text    |2    | rating_name|
|games      |releaseDate|date    |2    | releaseDate|
|games      |title     |text     |2    | title     |

The `field_type` column contains a type guess useful for inserting into a database. The `field_title` is the column heading in the CSV file or XLSX sheet, which is initially the same as the field_name.
After editing this file then you can rerun the transform:

```bash
flatterer games.json new_games_dir -f myfields.csv --only-fields
```

This can be useful for renameing columns, rearranging the field order or if you want to remove some fields the `--only-fields` flag will only include the fields in the edited file.

`datapackage.json` contains metadata in the [Tabular Datapackge Spec](https://specs.frictionlessdata.io/tabular-data-package/#language)

[More information on the output formats.](./outputs.md#outputs)

```{toctree}
:hidden:
options
library
outputs
changelog
development
```
