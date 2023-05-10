# Input Sources

JSON can be retrieved from a variety of sources, locally and on the web.  Below are examples of how to use the command line tool but they will also work with the 
[](./library.md#python-library).

## Local File

```bash
flatterer games.json games_dir
```
`games.json` can be a full or relative path to file.

## Stdin

Use `-` to get JSON from Stdin.

```bash
cat games.json | flatterer - games_dir
```

## HTTP(s)

```bash
flatterer 'https://example.com/my.json' games_dir
```

## S3

See [](./s3.md#s3) for how to configure connection to S3.

```bash
flatterer 's3://bucketname/path/in/bucket' games_dir
```
## GZIP 

If the file ends with `.gz` it is assumed to be a gzip compressed file. This will work with all input formats above.

```bash
flatterer 'games.json.gz' games_dir
```

## Multiple inputs

Multiple files can be selected. The last argument is the output directory and the rest are treated as inputs.

```
flatterer games.json games2.json games3.json games_dir
```

As long as the format of the input is the same in each file, you can get the JSON from various sources.
For example, this gets new line delimited json files (`ndjson`) from `stdin`, a gziped local file, a gzipped file on `s3` and a file on the web.

```bash
cat games.ndjson | flatterer --ndjson - games.ndjson.gz s3://bucketname/games.ndjson.gz http://example.com/games.ndjson games_dir
```
