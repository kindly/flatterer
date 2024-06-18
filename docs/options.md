# Option Reference

## Help <small> (CLI Only) </small>

```bash
flatterer --help
```

output looks like

``` 
Usage: flatterer [OPTIONS] [INPUTS]... OUTPUT_DIRECTORY

Options:
  --web                       Load web based version
  --csv / --nocsv             Output CSV files, default true
  --xlsx / --noxlsx           Output XLSX file, default false
  --sqlite / --nosqlite       Output sqlite.db file, default false
  --parquet / --noparquet     Output directory of parquet files, default false
  --postgres TEXT             Connection string to postgres. If supplied will
                              load data into postgres
  --sqlite-path TEXT          Output sqlite file to this file
  -d, --pushdown TEXT         Object keys and values, with this key name, will
                              be copied down to child tables
  -n, --no-link               Do not create `_link` fields
  -m, --main-table-name TEXT  Name of main table, defaults to name of the file
                              without the extension
  -p, --path TEXT             Key name of where json array starts, default top
                              level array
  -j, --ndjson                Is file a new line delemited JSON file, default
                              false
  --json-stream               File contains stream of json object, default
                              false
  --force                     Delete output directory if it exists, then run
                              command, default False
  -f, --fields TEXT           fields.csv file to use
  -o, --only-fields           Only output fields in fields.csv file
  -b, --tables TEXT           tables.csv file to use
  -l, --only-tables           Only output tables in tables.csv file
  -i, --inline-one-to-one     If array only has single item for all objects
                              treat as one-to-one
  -y, --arrays-new-table      Always treat arrays as a new tables, even when
                              they contain items that are not objects
  -s, --schema TEXT           JSONSchema file or URL to determine field order
  -t, --table-prefix TEXT     Prefix to add to all table names
  -a, --path-separator TEXT   Seperator to denote new path within the input
                              JSON. Defaults to `_`
  -h, --schema-titles TEXT    Use titles from JSONSchema in the given way.
                              Options are `full`, `slug`, `underscore_slug`.
                              Default to not using titles
  -w, --preview INTEGER       Only output this `preview` amount of lines in
                              final results
  --threads INTEGER           Number of threads, default 1, 0 means use number
                              of CPUs
  --json-path TEXT            JSON path within each object to use to filter
                              which objects to select
  --postgres-schema TEXT      When loading to postgres, put all tables into
                              this schema.
  --evolve                    When loading to postgres or sqlite, evolve
                              tables to fit data
  --drop                      When loading to postgres or sqlite, drop table
                              if already exists.
  --truncate                  When loading to postgres or sqlite, truncate table
                              if already exists.
  --id-prefix TEXT            Prefix for all `_link` id fields
  --stats                     Produce stats about the data in the
                              datapackage.json file
  --help                      Show this message and exit.
```

## Output Formats

**CSV:**  Defaults to outputing CSV in output directory `<OUTPUT_DIRECTORY>/csv/`.

**XLSX:**  Output xlsx file to `<OUTPUT_DIRECTORY>/output.xlsx`.

**SQLITE:**  Output sqlite file to  `<OUTPUT_DIRECTORY>/sqlite.db`.

**PARQUET:**  Output parquet files in `<OUTPUT_DIRECTORY>/parquet/`.

**POSTGRES:**  Output in database.

### CLI Usage

**Stop CSV output:**
```bash 
flatterer --nocsv INPUT_FILE OUTPUT_DIRECTORY
```

**xlsx output:**
```bash 
flatterer --xlsx INPUT_FILE OUTPUT_DIRECTORY
```

**sqlite output:**
```bash 
flatterer --sqlite INPUT_FILE OUTPUT_DIRECTORY
```

**parquet output:**
```bash 
flatterer --parquet INPUT_FILE OUTPUT_DIRECTORY
```

**postgres output:**
```bash 
flatterer --postgres='postgres://user:pass@host/dbname' INPUT_FILE OUTPUT_DIRECTORY
```

The connection string should be in [one of these formats](https://docs.rs/postgres/latest/postgres/config/struct.Config.html#examples). In addition, if you want the connection string from an environment variable then use the string `env` (for default DATABASE_URL enviroment variable) `env=MY_ENV_VAR` (for MY_ENV_VAR environment variable).

**postgres output from envirment variable:**
```bash 
flatterer --postgres='env=MY_ENV_VAR' INPUT_FILE OUTPUT_DIRECTORY
```
This will get the connection string from the `MY_ENV_VAR` environment variable.

### Python Usage

Export sqlite, xlsx, and parquet but not CSV.

```bash 
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', csv=False, sqlite=True, xlsx=True, parquet=True, postgres='postgres://user:pass@host/dbname')
```

## Main Table Name

Name of the table that represents data at the root of the JSON object.  

For CSV will create `<OUTPUT_DIRECTORY>/csv/<main_table_name>.csv` and for XLSX will be the first tab name.

For CLI defaults to name of input file without the file ending and for python defaults to `main`.



### CLI Usage

```bash 
flatterer -m games INPUT_FILE OUTPUT_DIRECTORY
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', main_table_name='games')
```

## Pushdown Fields

This allows you to copy values from top level object down to the child tables.  This is useful if you want to define your own join keys or if it is useful having certain values in all related tables, saving you doing extra joins for common queries.

You need to specify a list of fields names (keys in the JSON) that you want to appear on all one-to-many tables (child tables). The field will prefixed with the table name where the field existed and the value will be copied from that table.

For example if `main_table_name` is `game` and this is the input JSON:

```
[
  {
    "id": 4,
    "platforms": [
      {
        "name":"PC",
        "id": 1,
        "requirements": [
          {"ram": "4GB"}
        ] 
      }
    ]
  }
]
```

### CLI Usage

If `id` and `name` are pushdown fields:

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY -d id -d name
```

`platforms` table will contain:

|_link|_link_game|name|id|game_id|
|-----|----------|----|--|-------|
|0.platforms.0|0|PC|1|4|

As you can see a new column `game_id` is created containing the `id` from the `games` object.

`platforms_requirements` table will contain:

|_link|_link_platforms|_link_game|platforms_name|platforms_id|game_id|ram|
|-----|---------------|----------|--------------|----|--|-------|
|0.platforms.0.requirements.0|0.platforms.0|0|PC|1|4|4GB

This table also contains `game_id` but also `platforms_id` as `id` is pushed down from both parent tables.  Also as `name` is a pushdown field, `platforms_name` column is also created.


### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', pushdown=['id','name'])
```

## No Link Fields

Do not create any `_link` fields. This could be useful if `pushdown` pushes ids into 
child tables and you trust that this will be sufficient to link the tables back together.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY -n
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', pushdown=['id'], no_link=True)
```

## Path to JSON Array

Name of the object key where array of objects exists. Defaults to analysing the top level array.

Will not work with [](#json-stream) or [](#ndjson) option.

By default will work with:

```json
[
  {
    "id": 1,
    "title": "A Game",
    "releaseDate": "2015-01-01"
  },
  {
    "id": 2,
    "title": "B Game",
    "releaseDate": "2016-01-01"
  }
]

```  

If set to `games` will work with JSON like:

```json
{"games": [
    {
      "id": 1,
      "title": "A Game",
      "releaseDate": "2015-01-01"
    },
    {
      "id": 2,
      "title": "B Game",
      "releaseDate": "2016-01-01"
]}
```  

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY -p games
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', path='games')
```


## New Line Delemited JSON (NDJSON)

Input file is new line delimeted JSON.  This is the fastest input type as each line can be parsed in its own thread using `--threads`. 

```json
{"id": 1, "title": "A Game",  "releaseDate": "2015-01-01"}
{"id": 2,  "title": "B Game",  "releaseDate": "2016-01-01"}
```  

Will not work with [](#path-to-json-array) option.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --ndjson
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', ndjson=True)
```

## JSON Stream

Input file is stream of json objects, sometimes called Concatonated JSON.  Each object does not need to be on its own line. 

```json
{
  "id": 1,
  "title": "A Game",
  "releaseDate": "2015-01-01"
}
{
  "id": 2,
  "title": "B Game",
  "releaseDate": "2016-01-01"
}

```  

Will not work with [](#path-to-json-array) option.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --json-stream
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', json_stream=True)
```

## JSON Path Filter

This is used for filtering out objects to be included in the output, not for selecting values within an object. 

Use a [JSON path][https://goessner.net/articles/JsonPath/] expression to select if a particular object will be in output.

Flatterer will evaluate the JSON path expression against every object in the input, and if there is any non-null value, it will include that object in the result.

e.g


```json
[{
  "id": 1,
  "title": "A Film",
  "type": "film"
},
{
  "id": 2,
  "title": "A Game",
  "type": "game"
}]
```  

Using the above input JSON the following will only select object with `type = game` (only select the second object in the example).

### CLI Usage

```bash 
# careful as the $ needs escaping
flatterer INPUT_FILE OUTPUT_DIRECTORY --json-path "\$[?(@.type == 'game')]]" 
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', json_path="$[?(@.type == 'game')]")
```

More complicated expressions can be used including logical conditions so `$[?(@.type == 'game' || @.type == 'film')]` will select objects with either `type = 'game' OR type = 'film'`.


## Force

Delete the output folder if it exists before processing.


### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --force
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', force=True)
```

## Postgres Schema

Put tables into a postgres schema. Will create schema if it does not already exist.


### CLI Usage

```bash 
flatterer --postgres='postgres://user:pass@host/dbname' INPUT_FILE OUTPUT_DIRECTORY --postgres-schema=myschema
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', postgres='postgres://user:pass@host/dbname', postgres_schema='myschema')
```

## Evolve Tables

For postgres and sqlite. This will evolve the existing tables if the schema of the new flattened data is different. This is useful if you have new JSON data that comes in over time or you have lots of files you want to process one by one.

Evolving follows the following rules:

- If the new flattened data contains a table that does not exist in the database it will created.
- If the table already exists but the new data has extra fields, the table is altered to add the new fields.
- If table exists but fields that are in the database are not in the new data, they will result in nulls in the database when the new data is inserted. 
- If table exists and contains the same field name as the new data but the data types of the fields conflict:
  - For postgres the field is altered to being a `text` field so that both new and old data can exist (all types can be coerced to text).
  - For sqlite, as you can not alter existing types, the original type is kept. This will mean the data insertion will still work as sqlite treats any field as if it is text.  
- If no `id_prefix` is supplied random string will be added to `_link` fields so that these ids will be unique across loads. 

It is recommended to add an `id_prefix` that is unique for each JSON file load. It could conatain for example the name of the file or a date.

**Warning: this could mean you modify existing data.**

**Warning: Not completely parallel safe if multiple processes are inserting data into same database, it may cause an error if two processes are trying to add the same new field. This will not currupt any data and a retry should work.**


### CLI Usage

```bash 
flatterer --postgres='postgres://user:pass@host/dbname' INPUT_FILE OUTPUT_DIRECTORY --evolve
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', postgres='postgres://user:pass@host/dbname', evolve=True)
```

## Drop Tables

**Warning: this could mean you loose data**

For postgres and sqlite. Drop the existing table if it exists.

### CLI Usage

```bash 
flatterer --postgres='postgres://user:pass@host/dbname' --sqlite-path=sqlite.db INPUT_FILE OUTPUT_DIRECTORY --drop
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', postgres='postgres://user:pass@host/dbname', drop=True)
```

## Truncate Tables

**Warning: this could mean you loose data**

For postgres and sqlite. Truncate the existing table if it exists. This is useful if you want to load the data into a databse with the schema pre-defined.

### CLI Usage

```bash 
flatterer --postgres='postgres://user:pass@host/dbname' --sqlite-path=sqlite.db INPUT_FILE OUTPUT_DIRECTORY --truncate
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', postgres='postgres://user:pass@host/dbname', truncate=True)
```

## Fields File

Path to fields CSV file.  The fields file can be used for:

* Changing the field order in output files by rearranging the rows in the correct order. 
* Giving the fields a new name by using `field_title`
* Removing unwanted fields when using the [](#only-fields) option.

The CSV file needs the following headers: 

 * table_name
 * field_name

It has the optional heading of `field_title` which will default to the `field_name` if missing.

For example:

|table_name |field_name|field_type|count|field_title|
|-----------|----------|----------|-----|----------|
|platforms  |_link     |text      |3    |_link     |
|platforms  |_link_games|text     |3    |_link_games|
|platforms  |name      |text      |3    |name      |
|games      |_link     |text     |2    | _link     |
|games      |id        |number   |2    | id        |

It can have additional headers in the file but they will not be used. This is true of columns `count` and `field_type` in the above example.

Field order in the output will the same as the row order in the file.

`table_name` and `field_name` need to match up with the eventual structure of output. The easiest make sure of this is to edit the `fields.csv` that is in an output directory.  
You can generate just the fields.csv file by [not outputting the CSV files](#csv).

By default if there are fields in the data that are not in the `fields.csv` they will be added to the output after the defined fields.  Use [](#only-fields) to change this behaviour so that field not in the file will be excluded.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --fields fields.csv
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', fields='fields.csv')
```

## Only Fields

Only fields in the fields.csv file will be in the output.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --fields fields.csv --only-fields
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', fields='fields.csv', only_fields=True)
```

## Tables File

Path to tables CSV file.  The file can be used for:

* Changing the sheet order in xlsx output. 
* Giving the tables (and xlsx sheets) a new name by using `table_title`
* Removing unwanted tables when using the [](#only-tables) option.

The CSV file needs the following headers: 

 * table_name
 * table_title

For example:

|table_name |table_title|
|-----------|----------|
|platforms  |_link     |
|games      |_link     |

It can have additional headers in the file but they will not be used.

`tables_name` has to be the name that would be output by `flatterer`.  To make sure that these names are correct it is best to use the `tables.csv` that is always in the output directory as an basis for modifying the output.

By default if there are tables in the data that are not in the `tabless.csv` they will be added to the output after the defined tables.  Use [](#only-tables) to change this behaviour so that only tables in this file will be output.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --tables tables.csv
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', tables='tables.csv')
```

## Only Tables

Only tables in the tables.csv file will be in the output.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --tables tables.csv --only-tables
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', tables='tables.csv', only_tables=True)
```

## Inline One To One

When a key has an array of objects as its value, but that array only ever has single items in it, then treat it these single item as if they are a sub-object (not sub array).  
Without this set any array of objects will be treated like a one-to-many relationship and therefore have a new table associated with it.  With this set and if all arrays under a particular key only have one item in it, the child table will not be created and the values will appear in the parent table.  

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --inline-one-to-one
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', inline_one_to_one=True)

```

## Arrays as Table

Always create a new table for all arrays, even if they do not contain objects.  A new table is created with just the `_link` fields and a column named `value` which contains the value of the item in the array.  If the array item is a string that value will go in the output, for all other types the JSON encoded version of that type is included.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --arrays-as-table
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', arrays_as_table=True)

```

## Schema

Supply a JSONSchema file to help determine field ordering of the output.  If the schema supplied starts with `http` will try and download the schema from a remote server, otherwise it is assumed to be a file-system path.


### CLI Usage

For remote url:

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --schema https://example.com/schema.json
```

For local file:

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --schema schema.json
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', schema='https://example.com/schema.json')
```

## Table Prefix

Prefix to add to all table names. So if the output has a table called `mytable` and the "table prefix" is specified as `myprefix_` then the table (therefore csv file or excel sheet name) will be called `myprefix_mytable`.  
This can be useful if you are trying to namespace the output when inserting into an exiting database.


### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --table-prefix myprefix_
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', table_prefix='myprefix_')
```

## Path Separator

By default all table and field names have `_` as the seperator which denotes a that the field after the `_` is a sub-property. For example `myObject_myField` says that `myField` exists as a property of `myObject`. 

However some data already has `_` in the property names.  For example if in the input data theres was an object called `my_object` and the property called `my_field` then by default the field name would be `my_object_my_field`.  This is confusing as you might expect `object` to be a property of `my` and it might cause some name clashes.

To fix this you can change the path separator to whatever you like. You could choose `___` as a separator, so in the example above, the field would be called `my_object__my_field`.


### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --path-separator __
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', path_separoator='__')
```

## Schema Titles

When supplying a JSONSchema then use the `title` field in the schema for the field name (if it exists).  This option takes a string which can be one of:

* `full` Use make the `title` field from JSONSchema without modification.
* `slug` use the `title` field but slugify it removing all charactors that are non alphanumeric characters, lower casing and replaceing spaces with `-`.  For example `My *&*Strange   Title` will turn into `my-strange-title`
* `underscore_slug` same as slug but uses `_` instead of `-`. The previous example the output would be `my_strange_title`

If this option is left out or is the empty string then it will not take the heading from JSONSchema. 


### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --schema-titles underscore_slug
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', schema_titles='underscore_slug_')
```

## Stats

Adds additional statistics about the output files in the `datapackage.json` output.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --stats
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', stats=True)
```

## Preview

The number of rows written in final files. All statistics in `fields.csv` and `datapackage.json` will show counts related to *all* the data.

### CLI Usage

Only output first 10 lines of all the tables.

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --preview 10
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', preview=10)
```

## Threads

The number of threads used to process the data. Default to 1. If set to 0 will use amount of CPUs.

Works best with new line delimited JSON `--ndjson` as JSON parsing can then be done by each thread. This can about a x3 times improvement with 6 threads if you have that many CPU cores. Without `--ndjson` makes only about x1.24 improvement on 2 threads and not worth going over 2 as it will not lead to performance improvement. For very small datasets (less than 100 object) using threads will most likely be slower.

**Warning:** When using this mode, not checks will be done to ensure an array of objects exists in the data. So in some circumstances, if the wrong options are chosen, no error will be raised.  

**Warning:** May have issues with inline-one-to-one as each thread will determine what should be inlined.

### CLI Usage

```bash 
flatterer INPUT_FILE OUTPUT_DIRECTORY --threads 0
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', threads=10)
```

## Sql Scripts

Python only. Export scripts for importing data into the database.

### Python Usage

flatterer.flatten('inputfile.json', 'ouput_dir', sql_scripts=True)


## Low Memory (api only)

Reduces memory usage, sacrificing some speed.  Use this if JSON contains very large JSON objects.

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', low_memory=True)
```