# Option Reference

## Help <small> (CLI Only) </small>

```bash
flatterer --help
```

output looks like

``` 
Usage: flatterer [OPTIONS] INPUT_FILE OUTPUT_DIRECTORY

Options:
  --csv / --nocsv             Output CSV files, default true
  --xlsx / --noxlsx           Output XLSX file, default false
  -m, --main-table-name TEXT  Name of main table, defaults to name of the file
                              without the extension
  -p, --path TEXT             Key name of where json array starts, default top
                              level array
  -j, --json-lines            Is file a jsonlines file, default false
  --force                     Delete output directory if it exists, then run
                              command, default False
  -f, --fields TEXT           fields.csv file to use
  -o, --only-fields           Only output fields in fields.csv file
  -i, --inline-one-to-one     If array only has single item for all objects
                              treat as one-to-one
  -s, --schema TEXT           JSONSchema file or URL to determine field order
  -t, --table-prefix TEXT     Prefix to add to all table names
  -a, --path-separator TEXT   Seperator to denote new path within the input
                              JSON. Defaults to `_`
  -i, --schema-titles TEXT    Use titles from JSONSchema in the given way.
                              Options are `full`, `slug`, `underscore_slug`. Default to
                              not using titles.
  --help                      Show this message and exit.
```

## CSV

Output CSV files in output directory `<OUTPUT_DIRECTORY>/csv/`.
Defaults to True


### CLI Usage

```bash 
flatterer --nocsv INPUT_FILE OUTPUT_DIRECTORY
```

### Python Usage

```bash 
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', csv=False)
```


## XLSX

Output XLSX file in output directory `<OUTPUT_DIRECTORY>/output.xlsx`. Defaults to False


### CLI Usage

Main table name set to `games`

```bash 
flatterer --xlsx INPUT_FILE OUTPUT_DIRECTORY
```

### Python Usage

```bash 
import flatterer

flatterer.flatten('inputfile.json', 'ouput_dir', xlsx=True)
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


## Path to JSON Array

Name of the object key where array of objects exists. Defaults to analysing the top level array.

Will not work with [](#json-lines) option.

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


## JSON Lines

Make flatterer accept new-line-delimeted JSON as well as any stream supported by [serde_json](https://docs.serde.rs/serde_json/struct.StreamDeserializer.html). eg:


```json
{"id": 1, "title": "A Game",  "releaseDate": "2015-01-01"}
{"id": 2,  "title": "B Game",  "releaseDate": "2016-01-01"}

```  

OR 

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
flatterer INPUT_FILE OUTPUT_DIRECTORY --json-lines
```

### Python Usage

```python
import flatterer

flatterer.flatten('inputfile.jl', 'ouput_dir', json_lines=True)
```

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

## Fields File

Path to fields csv file.  The CSV file needs the following headers: 

 * table_name
 * field_name
 * field_type

It has the optional heading of `field_title` which will default to the `field_name` if missing.

For example:

|table_name |field_name|field_type|count|field_title|
|-----------|----------|----------|-----|----------|
|platforms  |_link     |text      |3    |_link     |
|platforms  |_link_games|text     |3    |_link_games|
|platforms  |name      |text      |3    |name      |
|games      |_link     |text     |2    | _link     |
|games      |_link_games|text    |2    | _link_games|
|games      |id        |number   |2    | id        |

It can have additional headers in the file but they will not be used.

Field order in the output will the same as the row order in the the file.

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

flatterer.flatten('inputfile.jl', 'ouput_dir', inline_one_to_one=True)

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
