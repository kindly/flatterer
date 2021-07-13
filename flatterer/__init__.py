import decimal

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
    path='',
    main_table_name='main',
    emit_path=[],
    json_lines=False,
    force=False
):
    flatterer.flatten_rs(input, output_dir, csv, xlsx, path, main_table_name, emit_path, json_lines, force)


def iterator_flatten(
    iterator,
    output_dir,
    csv=True,
    xlsx=False,
    main_table_name='main',
    emit_path=[],
    force=False
):

    flatterer.iterator_flatten_rs(bytes_generator(iterator), output_dir, csv, xlsx, main_table_name, emit_path, force)

