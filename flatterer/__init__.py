import decimal
import click

import orjson
import ijson

from .flatterer import iterator_flatten_rs, flatten_rs


def default(obj):
    if isinstance(obj, decimal.Decimal):
        return str(obj)
    raise TypeError


def bytes_generator(iterator):
    for item in iterator:
        if isinstance(item, bytes):
            yield item
        if isinstance(item, str):
            yield str.encode()
        if isinstance(item, dict):
            yield orjson.dumps(item, default=default)


def flatten(
    input,
    output_dir,
    csv=True,
    xlsx=False,
    path="",
    main_table_name="main",
    emit_path=[],
    json_lines=False,
    force=False,
):
    flatterer.flatten_rs(
        input,
        output_dir,
        csv,
        xlsx,
        path,
        main_table_name,
        emit_path,
        json_lines,
        force,
    )


def iterator_flatten(
    iterator,
    output_dir,
    csv=True,
    xlsx=False,
    main_table_name="main",
    emit_path=[],
    force=False,
):

    flatterer.iterator_flatten_rs(
        bytes_generator(iterator),
        output_dir,
        csv,
        xlsx,
        main_table_name,
        emit_path,
        force,
    )


@click.command()
@click.option("--csv/--nocsv", default=True, help="Output CSV default true")
@click.option("--xlsx/--noxlsx", default=False, help="Output XLSX file default false")
@click.option("--main-table-name", "-m", default="main", help="Name of main table")
@click.option(
    "--path",
    "-p",
    default="",
    help="Key name of where json array starts, default look for top level array",
)
@click.option(
    "--json-lines",
    "-j",
    is_flag=True,
    default=False,
    help="Is file a jsonlines file. Will ignore path if so",
)
@click.option(
    "--force",
    "-f",
    is_flag=True,
    default=False,
    help="Delete output directory if it exists, then run command",
)
@click.argument("input_file")
@click.argument("output_directory")
def cli(
    input_file,
    output_directory,
    csv=True,
    xlsx=False,
    path="",
    main_table_name="main",
    json_lines=False,
    force=False,
):

    flatten(
        input_file,
        output_directory,
        csv=csv,
        xlsx=xlsx,
        path=path,
        main_table_name=main_table_name,
        json_lines=json_lines,
        force=force,
    )
