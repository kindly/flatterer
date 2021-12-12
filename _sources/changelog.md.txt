# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

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

