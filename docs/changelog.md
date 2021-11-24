# Change Log
All notable changes to this project will be documented in this file.
 
The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).
 

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

