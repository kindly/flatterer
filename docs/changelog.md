# Change Log
All notable changes to this project will be documented in this file.


and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.20.0] - 2024-10-19

### New
- Use rust_xlsxwriter to write xlsx files.

## [0.19.19] - 2024-07-07

### Fixed
- Parquet headings now from fields.csv
- text fields are kept as text in dataframe

## [0.19.18] - 2024-06-22

### Fixed
- Intermediate tables excluded using fields.csv or tables.csv with fields_only and tables_only will not cause error.

## [0.19.17] - 2024-06-18

### New
- truncate postgres

### Fixed
- timezone date types now accepted in postgres

## [0.19.15] - 2024-05-09

### Fixed
- nan/inf ignored for xlsx as causing crash

## [0.19.14] - 2024-02-10

### Fixed
- Large xlsx cell values being truncated panic when multi threading.

## [0.19.13] - 2024-01-26

### Fixed
- Large xlsx cell values being truncated causing panic if in unicode char.

## [0.19.12] - 2023-12-03

### New
- Upgrade deps, low_memory option for API

## [0.19.10] - 2023-10-02

### New
- Upgrade deps, better build times due to latest duckdb

## [0.19.8] - 2023-06-21

### New
- `arrays_as_table` option added to convert all arrays to their own table.

## [0.19.6] - 2023-06-21

### Fixed
- Errors get raised for postgresql conversion.

## [0.19.5] - 2023-06-07

### Fixed
- Parquet naming of headers where incorrect for dates.

## [0.19.4] - 2023-05-26

### Fixed
- Allow multiple files while downloading from s3
- Stop detecting floats where precision is too low. 

## [0.19.3] - 2023-05-11
### Fixed
- CSV output to S3 broken in some cases.
- Stop csv directory being made when using S3


## [0.19.1] - 2023-05-10

### New
- JSON Input sources from STDIN, HTTP, S3 and allow all inputs to be GZIPed if have `.gz` ending.
- Command line now accepts multiple files from any source.
- S3 output for CSV and PARQUET that is streamed from the input.
- `json-path` query to filter JSON objects.
- `stats` produce statistics on ouput files. 

## [0.18.0] - 2023-03-17

### New
- Better type guessing for database inserts.
- `no_link` option that removes `_link` fields in the output.

## [0.17.1] - 2023-01-07

### Fixed 
- Truncate cell that is larger than xlsx allows.
- Allow more rows in xlsx in non threaded mode.

## [0.17.0] - 2022-12-03

### New
- Web Assembly version of libflatterer. Available to use here https://lite.flatterer.dev/.
- Upgrade to vue 3 and vite for web frontend.

### Fixed 
- Ignore blank lines in json lines files
- Better errors when too many files are open
- Better errors when file could be json-stream

## [0.16.2] - 2022-10-25

### New
- Support python 3.11

## [0.16.2] - 2022-10-16

### Fixed 
- Error not writing larger XLSX files

## [0.16.1] - 2022-10-16

### Fixed 
- Cors for web api

## [0.16.0] - 2022-07-27

### New
- Local web interface for exploring flatterer features `flatterer --web`. 

## [0.15.0] - 2022-07-27

### New
- `evolve` option for sqlite and postgres.  Can add data to existing tables and will alter tables if new fields are needed.
- `drop` option for `sqlite`.

### Changed
- No need for output directory when using `sqlite` or `postgres` outputs . Will use temp space if not supplied.
- `sqlite-path` option will switch on `sqlite` option.

## [0.14.2] - 2022-07-27

### New
- Postgres connection from environment variable

## [0.14.1] - 2022-07-23

### Fixed
- `sql_script` option to export scripts that for sqlite and postgres to make output backward compatable with earlier versions.

## [0.14.0] - 2022-07-22

### New
- `pushdown` option.  Copy data from top level objects down to child (one-to-many) tables.  This is useful if the data has its own keys (such as `id` fields) that you want to exist in the related tables. Also useful for denormalizing the data so querying on a common field, requires less joining.
  
- `postgres` option.  Export to postgres database by supplying a connection string.

- `postgres-schema` option. Choose a postgres schema to insert data into. 

- `drop` option. If table already exists.

### Removed

- sqlite and postgres scripts in output directory. No longer required as the actual database tables can be created by

## [0.13.2] - 2022-06-24

### New
- `files` option, so multiple files can be supplied at once.

## [0.13.1] - 2022-05-27

- Threads option now can output xlsx

## [0.13.0] - 2022-04-28

### New

- Threads option, so that can be run on all cores. Works best with ndjson input.
- Parquet export option.

### Changed

- BREAKING: `json-lines` option renamed to `ndjson` 
- New `json-stream` option that works in same way as the old json-lines option and accepts concatonated json.

## [0.12.12] - 2022-04-15

### Changed

- Datapackage output uses correct date type
- Lists of strings are now escaped the same way as optional quoted CSVs

## [0.12.11] - 2022-04-09

### Changed

- Clearer errors when error happens in rust. BREAKING CHANGE, if catching certain error types in python these may have changed.
- datapackage output now has forign keys.


## [0.12.10] - 2022-03-21

### Fixed

- Python decimal converted to float not string.


## [0.12.9] - 2022-03-07

### Fixed

- SQLite export lower memory use


## [0.12.8] - 2022-03-01

### Changed

- SQLite export has indexes and foreign key contraints.


## [0.12.7] - 2022-02-27

### Fixed

- main_table_name was number caused exception


## [0.12.6] - 2022-02-26

### Fixed

- list of JSON strings supplied to `flatten` fixed.
- datapackage.json named correctly.

## [0.12.4] - 2022-02-18

### Changed

- `flatten` python function now accepts iterator
- Docs for `flatten`
- Tests for `flatten`
- iterator_flatten now deprecated as it is just a subset of `flatten'

## [0.12.3] - 2022-02-15

### Fixed

- More lenient if tmp directory can not be deleted.

## [0.12.1] - 2022-02-15

### Fixed

- Preview option in python CLI and library.


## [0.12] - 2022-02-02

### New

- SQLite export option

## [0.11] - 2022-01-25

### New

- Support top level object. All list of objects are streamed and top level object data saved in main table.
- New yaglish parser for both json stream and arrays.
- Library has schema_guess function to tell if data is a JSON Stream or has an array of object.

### Fixed

- Empty objects do not make a line in output.


## [0.10.1] - 2021-12-22

### New

- ctrlc support added.

### Changed

- Logging output improved.
- Traceback not shown for CLI use.

### Fixed

- Occurences where output folder not being deleted.

## [0.10] - 2021-12-22

### New

- `tables.csv` input in order to control tab names. [Tables File option](https://flatterer.opendata.coop/options.html#tables-file)
- Beginning to use logging.


### Changed

- Better handling of long excel sheet names names. See https://github.com/kindly/flatterer/issues/12
- `field_type` no longer required in fields.csv.
- All `_link` fields have are 0 indexed.
- Removal of redundant `_link_<main_table>` from main table.
- More human readable error messages.


### Fixed

- Bad characters in XLSX stripped and raise warning. 
- Check limits on XLSX files and raise error if found.
- Lots of edge cases handled better. See https://github.com/kindly/flatterer/issues/17
- All errors will remove output directory instead of leaving unusable directory or partial data.

## [0.9] - 2021-12-12
 
### Fixed

- Removed unwrap on channel send, to remove possible panic.

### Changed

- Table ordering of output in JSON input order.  Making `xlsx` and `fields.csv` table order reflect the input data.
- Lib has new `FlatFiles::new_with_dafualts()` to make using the library less verbose.
- Use insta for more tests.

### New

- Lib has preview option, meaning CSV output will optionally only show specified number of lines.

## [0.8] - 2021-12-01
 
### Changed

- Paths to data in sqlite and postgres start at root of output.
- Clippy for linting and insta for tests.
 
## [0.7.1] - 2021-12-01
 
### Changed

- Do less work when just exporting Metadata.
- Minor speedup due to not using `format` so much.
 

## [0.7] - 2021-11-24
 
### Added 
 
-  Ability to add `field_title` to `fields.csv` so you can rename column headings.
-  `schema-titles` option to get field names out of JSONSchema.
-  Speed improvments using `SmartString` and `smallvec`.


## [0.6.2] - 2021-11-24
 
### Fixed
 
-  Change to pypi metadata
-  Tests run in action
-  Changelog in docs


## [0.6.1] - 2021-11-24
 
### Fixed
 
-  Regression in speed due to new error handling


## [0.6] - 2021-11-23

### Changed

- New error handling using `anyhow`, giving errors more context.


## [0.5] - 2021-11-22

### Added

- [Schema option](https://flatterer.opendata.coop/options.html#schema) to supply JSONSchema, to make field order the same as schema. 
- [Table prefix option](https://flatterer.opendata.coop/options.html#table-prefix) to namespece exported tables.
- [Path separator option](https://flatterer.opendata.coop/options.html#path-separator) to change the JSON path separator from `_` if the data has them in the field names.


## [0.4] - 2021-11-20
 
### Added

-  Postgresql and sqlite scripts to load CSV data into databases.
-  Wheel builds for Windows and MacOS, automatically published using github actions.

 
## [0.3] - 2021-10-25
 
### Added

- [Inline One to One option](https://flatterer.opendata.coop/options.html#inline-one-to-one) to mean that if an array only has one item in for all the data then treat it as sub-object.

