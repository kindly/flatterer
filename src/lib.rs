use crossbeam_channel::bounded;
use libflatterer::{flatten, flatten_from_jl, FlatFiles, Selector};
use serde_json::Value;
use std::thread;

use pyo3::prelude::*;
use pyo3::types::PyIterator;
use std::fs::File;
use std::io::BufReader;
use env_logger::Env;

#[pymodule]
fn flatterer(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn setup_logging(
        _py: Python,
        default_log_level: String,
    ) {
        env_logger::Builder::from_env(Env::new().filter_or("FLATTERER_LOG", &default_log_level)).init();
    }
    #[pyfn(m)]
    fn flatten_rs(
        _py: Python,
        input_file: String,
        output_dir: String,
        csv: bool,
        xlsx: bool,
        path: String,
        main_table_name: String,
        emit_path: Vec<Vec<String>>,
        json_lines: bool,
        force: bool,
        fields: String,
        only_fields: bool,
        tables: String,
        only_tables: bool,
        inline_one_to_one: bool,
        schema: String,
        table_prefix: String,
        path_separator: String,
        schema_titles: String,
    ) -> PyResult<()> {
        let flat_files_res = FlatFiles::new(
            output_dir,
            csv,
            xlsx,
            force,
            main_table_name,
            emit_path,
            inline_one_to_one,
            schema,
            table_prefix,
            path_separator,
            schema_titles,
        );

        let mut selectors = vec![];

        if !path.is_empty() {
            selectors.push(Selector::Identifier(format!("\"{}\"", path)));
        }

        if let Err(err) = flat_files_res {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                err.to_string(),
            ));
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if !fields.is_empty() {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                    err.to_string(),
                ));
            }
        }
        if !tables.is_empty() {
            if let Err(err) = flat_files.use_tables_csv(tables, only_tables) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                    err.to_string(),
                ));
            }
        }

        let file;

        match File::open(&input_file) {
            Ok(input) => {
                file = BufReader::new(input);
            }
            Err(err) => {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "Can not open file `{}`: {}",
                    input_file,
                    anyhow::Error::new(err)
                )));
            }
        };

        if json_lines {
            if let Err(err) = flatten_from_jl(file, flat_files) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{}",
                    err
                )));
            };
        } else {
            if let Err(err) = flatten(file, flat_files, selectors) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{}",
                    err
                )));
            };
        };
        Ok(())
    }

    #[pyfn(m)]
    fn iterator_flatten_rs(
        py: Python,
        mut objs: &PyIterator,
        output_dir: String,
        csv: bool,
        xlsx: bool,
        main_table_name: String,
        emit_path: Vec<Vec<String>>,
        force: bool,
        fields: String,
        only_fields: bool,
        tables: String,
        only_tables: bool,
        inline_one_to_one: bool,
        schema: String,
        table_prefix: String,
        path_separator: String,
        schema_titles: String,
    ) -> PyResult<()> {
        let flat_files_res = FlatFiles::new(
            output_dir,
            csv,
            xlsx,
            force,
            main_table_name,
            emit_path,
            inline_one_to_one,
            schema,
            table_prefix,
            path_separator,
            schema_titles,
        );

        if let Err(err) = flat_files_res {
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "{:?}",
                err
            )));
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if !fields.is_empty() {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{}",
                    err
                )));
            }
        }

        if !tables.is_empty() {
            if let Err(err) = flat_files.use_tables_csv(tables, only_tables) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{}",
                    err
                )));
            }
        }

        let (sender, receiver) = bounded(1000);

        let handler = thread::spawn(move || -> PyResult<()> {
            for value in receiver {
                flat_files.process_value(value);
                if let Err(err) = flat_files.create_rows() {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                        err.to_string(),
                    ));
                }
            }

            if let Err(err) = flat_files.write_files() {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                    err.to_string(),
                ));
            }
            Ok(())
        });

        let mut gilpool;

        loop {
            unsafe {
                gilpool = py.new_pool();
            }

            let obj = objs.next();
            if obj.is_none() {
                break;
            }

            let result = obj.unwrap(); //checked for none

            let json_bytes = PyAny::extract::<&[u8]>(result?)?;

            match serde_json::from_slice::<Value>(json_bytes) {
                Ok(value) => {
                    if let Err(err) = sender.send(value) {
                        return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                            err.to_string(),
                        ));
                    }
                }
                Err(err) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                        err.to_string(),
                    ))
                }
            }

            drop(gilpool)
        }

        drop(sender);

        match handler.join() {
            Ok(result) => {
                if let Err(err) = result {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(
                        err.to_string(),
                    ));
                }
            }
            Err(err) => {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )))
            }
        }
        Ok(())
    }

    Ok(())
}
