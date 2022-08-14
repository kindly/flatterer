import unittest
import json

import flatterer
import ijson
import pandas

def jl_item_generator(filename):
    with open(filename, 'rb') as f:
        for item in ijson.items(f,'', multiple_values=True):
            yield item

def line_generator(filename):
    with open(filename, 'rb') as f:
        for line in f:
            yield line

def array_item_generator():
    with open('fixtures/basic.json', 'rb') as f:
        for item in ijson.items(f,'item'):
            yield item

class TestBasic(unittest.TestCase):

    def check_output(self, output, directory='fixtures/basic_expected'):

        for field in (
            'fields',
            'tables',
        ):
            self.assertEqual(
                output[field].to_dict('records'), 
                pandas.read_csv(f'{directory}/{field}.csv').to_dict('records'), 
            )

        for table in (
            'main',
            'platforms',
            'developer',
        ):
            self.assertEqual(
                output['data'][table].to_dict('records'), 
                pandas.read_csv(f'{directory}/csv/{table}.csv').to_dict('records'), 
            )


    def test_array_top(self):
        output = flatterer.flatten(['fixtures/basic.json'], dataframe=True, files=True)
        self.check_output(output)

    def test_jsonlines(self):
        output = flatterer.flatten('fixtures/basic.jl', dataframe=True, json_stream=True)
        self.check_output(output)

    def test_jl_iterator_basic(self):
        output = flatterer.flatten(jl_item_generator('fixtures/basic.jl'), dataframe=True)
        self.check_output(output)

    def test_jl_iterator_large(self):
        output = flatterer.flatten(line_generator('fixtures/daily_16.json'), force=True, threads=0, dataframe=True)
        self.assertEqual(len(output['data']['main']), 4999)

    def test_array_iterator(self):
        output = flatterer.flatten(array_item_generator(), dataframe=True)
        self.check_output(output)

    def test_python_list(self):
        with open('fixtures/basic.json') as f:
            item_list = json.load(f)
        output = flatterer.flatten(item_list, dataframe=True)
        self.check_output(output)

    def test_python_list_of_strings(self):
        with open('fixtures/basic.json') as f:
            item_list = json.load(f)
        string_list = [json.dumps(item) for item in item_list]
        output = flatterer.flatten(string_list, dataframe=True)
        self.check_output(output)

    def test_python_list_of_bytes(self):
        with open('fixtures/basic.json') as f:
            item_list = json.load(f)
        bytes_list = [json.dumps(item).encode() for item in item_list]
        output = flatterer.flatten(bytes_list, dataframe=True)
        self.check_output(output)

    def test_pushdown(self):
        output = flatterer.flatten('fixtures/basic.json', dataframe=True, pushdown=['id', 'title'])
        self.check_output(output, 'fixtures/pushdown_expected')

    def test_multiple(self):
        output = flatterer.flatten(['fixtures/basic.json', 'fixtures/basic.json'], dataframe=True, files=True)

        df = pandas.read_csv(f'fixtures/basic_expected/fields.csv')
        df['count'] = df['count'] * 2

        self.assertEqual(
            output['fields'].to_dict('records'), 
            df.to_dict('records'), 
        )
