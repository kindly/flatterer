# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [0.12.1] - 2021-02-15

### Fixed

- Preview option in python CLI and library.


## [0.12] - 2021-02-02

### New

- SQLite export option

## [0.11] - 2021-01-25

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

