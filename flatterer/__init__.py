import decimal
import click
import os.path
import shutil
import tempfile

import orjson
import pandas

from .flatterer import iterator_flatten_rs, flatten_rs, setup_logging, setup_ctrlc, web_rs

LOGGING_SETUP = False


def pretty_dict(output, indent=0, key=""):
    indent_str = " " * indent
    if key:
        key = f'"{key}": '
    yield indent_str + key + "{"

    for key, value in output.items():
        if isinstance(value, dict):
            yield from pretty_dict(value, indent+4, key)
            continue
        value_str = f'File Path - {value}'
        if isinstance(value, pandas.DataFrame):
            fields = ", ".join(value.keys())
            if len(fields) > 40:
                fields = f'{fields[:40]}... ({len(value.keys())} fields)'

            value_str = "DataFrame - " + fields
        yield f'{indent_str}    "{key}": "{value_str}"'

    yield indent_str + "}"


class PrettyDict(dict):
    def __repr__(self):
        return "\n".join(pretty_dict(self))


def default(obj):
    if isinstance(obj, decimal.Decimal):
        return float(obj)
    raise TypeError


def bytes_generator(iterator):
    for item in iterator:
        if isinstance(item, bytes):
            yield item
        if isinstance(item, str):
            yield item.encode('utf-8')
        if isinstance(item, dict):
            yield orjson.dumps(item, default=default)


def flatten(
    input,
    output_dir='',
    csv=True,
    xlsx=False,
    sqlite=False,
    parquet=False,
    dataframe=False,
    path=[],
    main_table_name='main',
    emit_obj=[],
    ndjson=False,
    json_stream=False,
    force=False,
    fields_csv='',
    only_fields=False,
    tables_csv='',
    only_tables=False,
    inline_one_to_one=False,
    schema="",
    id_prefix="",
    table_prefix="",
    path_separator="_",
    schema_titles="",
    sqlite_path="",
    preview=0,
    threads=1,
    files=False,
    log_error=False,
    postgres="",
    postgres_schema="",
    drop=False,
    pushdown=[],
    sql_scripts=False,
    evolve=False,
):
    global LOGGING_SETUP
    if not LOGGING_SETUP:
        setup_logging("warning")
        LOGGING_SETUP = True
    
    using_tmp = False
    
    if not output_dir:
        if not dataframe and not sqlite_path and not postgres:
            raise AttributeError("Please set an `output_dir` or set `dataframe=True` or `postgres` or `sqlite-db`")
        output_dir = tempfile.mkdtemp(prefix="flatterer-")
        force = True
        using_tmp = True
    
    if dataframe:
        csv = True
    
    method = None
    try:
        try:
            if isinstance(input, str):
                method = 'flatten'
                input = [input]
            else:
                iter(input)
                if not files:
                    method = 'iter'
                else:
                    input = list(input)
                    method = 'flatten'
        except TypeError:
            input = [input]
            method = 'flatten'
        
        if sqlite_path:
            sqlite = True
        
        if method == 'flatten':
            flatten_rs(input, output_dir, csv, xlsx, sqlite, parquet,
                       main_table_name, tables_csv, only_tables, fields_csv, only_fields,
                       inline_one_to_one, path_separator, preview, 
                       table_prefix, id_prefix, emit_obj, force,  
                       schema, schema_titles, path, json_stream, ndjson, 
                       sqlite_path, threads, log_error, postgres, postgres_schema, drop, pushdown, sql_scripts, evolve)
        elif method == 'iter':
            if path:
                raise AttributeError("path not allowed when supplying an iterator")
            iterator_flatten_rs(bytes_generator(input), output_dir, csv, xlsx, sqlite, parquet,
                       main_table_name, tables_csv, only_tables, fields_csv, only_fields,
                       inline_one_to_one, path_separator, preview, 
                       table_prefix, id_prefix, emit_obj, force,  
                       schema, schema_titles, sqlite_path, threads, log_error, 
                       postgres, postgres_schema, drop, pushdown, sql_scripts, evolve)
        else:
            raise AttributeError("input needs to be a string or a generator of strings, dicts or bytes")

        output = PrettyDict(
            fields=pandas.read_csv(os.path.join(output_dir, 'fields.csv')),
            tables=pandas.read_csv(os.path.join(output_dir, 'tables.csv')),
            data=PrettyDict()
        )

        if csv:
            for table in output['tables']['table_title']:
                csv_path = os.path.join(output_dir, 'csv', str(table) + '.csv')
                if dataframe:
                    output['data'][table] = pandas.read_csv(csv_path)
                else:
                    output['data'][table] = csv_path
        
        if sqlite:
            output['sqlite'] = os.path.join(output_dir, 'sqlite.db')

        if xlsx:
            output['xlsx'] = os.path.join(output_dir, 'output.xlsx')

        return output

    finally:
        if using_tmp:
            shutil.rmtree(output_dir)



def iterator_flatten(*args, **kw):
    flatten(*args, **kw)


@click.command()
@click.option('--web', default=False, is_flag=True ,help='Load web based version')
@click.option('--csv/--nocsv', default=True, help='Output CSV files, default true')
@click.option('--xlsx/--noxlsx', default=False, help='Output XLSX file, default false')
@click.option('--sqlite/--nosqlite', default=False, help='Output sqlite.db file, default false')
@click.option('--parquet/--noparquet', default=False, help='Output directory of parquet files, default false')
@click.option('--postgres', default="", help='Connection string to postgres. If supplied will load data into postgres')
@click.option('--sqlite-path', default="", help='Output sqlite file to this file')
@click.option('--pushdown', '-d', multiple=True, help='Object keys and values, with this key name, will be copied down to child tables')
@click.option('--main-table-name', '-m', default=None,
              help='Name of main table, defaults to name of the file without the extension')
@click.option('--path', '-p', default='', help='Key name of where json array starts, default top level array')
@click.option('--ndjson', '-j', is_flag=True, default=False,
              help='Is file a new line delemited JSON file, default false')
@click.option('--json-stream', is_flag=True, default=False,
              help='File contains stream of json object, default false')
@click.option('--force', is_flag=True, default=False,
              help='Delete output directory if it exists, then run command, default False')
@click.option('--fields', '-f', default="", help='fields.csv file to use')
@click.option('--only-fields', '-o', is_flag=True, default=False, help='Only output fields in fields.csv file')
@click.option('--tables', '-b', default="", help='tables.csv file to use')
@click.option('--only-tables', '-l', is_flag=True, default=False, help='Only output tables in tables.csv file')
@click.option('--inline-one-to-one', '-i', is_flag=True, default=False,
              help='If array only has single item for all objects treat as one-to-one')
@click.option('--schema', '-s', default="",
              help='JSONSchema file or URL to determine field order')
@click.option('--table-prefix', '-t', default="",
              help='Prefix to add to all table names')
@click.option('--path-separator', '-a', default="_",
              help='Seperator to denote new path within the input JSON. Defaults to `_`')
@click.option('--schema-titles', '-h', default="",
              help='Use titles from JSONSchema in the given way. Options are `full`, `slug`, `underscore_slug`. Default to not using titles')
@click.option('--preview', '-w', default=0,
              help='Only output this `preview` amount of lines in final results')
@click.option('--threads', default=1,
              help='Number of threads, default 1, 0 means use number of CPUs')
@click.option('--postgres-schema', default="", help='When loading to postgres, put all tables into this schema.')
@click.option('--evolve', is_flag=True, default=False, help='When loading to postgres or sqlite, evolve tables to fit data')
@click.option('--drop', is_flag=True, default=False, help='When loading to postgres or sqlite, drop table if already exists.')
@click.option('--id-prefix', default="", help='Prefix for all `_link` id fields')
@click.argument('input_file', required=False)
@click.argument('output_directory', required=False)
def cli(
    input_file,
    output_directory,
    web=False,
    csv=True,
    xlsx=False,
    sqlite=False,
    sqlite_path='',
    parquet=False,
    postgres='',
    path='',
    main_table_name=None,
    ndjson=False,
    json_stream=False,
    force=False,
    fields="",
    only_fields=False,
    tables="",
    only_tables=False,
    inline_one_to_one=False,
    schema="",
    table_prefix="",
    path_separator="_",
    schema_titles="",
    preview=0,
    threads=1,
    postgres_schema="",
    evolve=False,
    drop=False,
    pushdown=[],
    id_prefix=""
):
    if web:
        import pathlib
        filepath = pathlib.Path(__file__).resolve().parent
        os.environ['STATIC_FILES'] = str(filepath / 'static')
        if not os.environ.get('NO_BROWSER', ''):
            os.environ['OPEN_BROWSER'] = "true"

        os.environ['CLEAN_TMP_TIME'] = "0"
        setup_ctrlc()
        web_rs()
        return
    
    if not input_file:
        click.echo("An input file is needed as first argument.")
        raise click.Abort

    global LOGGING_SETUP
    if not LOGGING_SETUP:
        setup_logging("info")
        LOGGING_SETUP = True
        setup_ctrlc()
    

    if not main_table_name:
        main_table_name = input_file.split('/')[-1].split('.')[0]
    
    path_list = []
    if path:
        path_list.append(path)

    try:
        flatten(input_file,
                output_directory,
                csv=csv,
                xlsx=xlsx,
                sqlite=sqlite,
                parquet=parquet,
                postgres=postgres,
                path=path_list,
                main_table_name=main_table_name,
                ndjson=ndjson,
                json_stream=json_stream,
                force=force,
                fields_csv=fields,
                only_fields=only_fields,
                tables_csv=tables,
                only_tables=only_tables,
                inline_one_to_one=inline_one_to_one,
                schema=schema,
                table_prefix=table_prefix,
                path_separator=path_separator,
                schema_titles=schema_titles,
                preview=preview,
                threads=threads,
                log_error=True,
                postgres_schema=postgres_schema,
                evolve=evolve,
                drop=drop,
                pushdown=pushdown,
                id_prefix=id_prefix,
                sqlite_path=sqlite_path)
    except IOError:
        pass
