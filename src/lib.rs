use crossbeam_channel::bounded;
use eyre::{Result, WrapErr};
use libflatterer::{flatten, FlatFiles, TERMINATE};
use serde_json::Value;
use std::thread;

use env_logger::Env;
use pyo3::prelude::*;
use pyo3::types::PyIterator;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::Ordering;

#[pymodule]
fn flatterer(_py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn setup_ctrlc(_py: Python) {
        log::debug!("ctrlc setup");
        ctrlc::set_handler(|| {
            log::debug!("ctrlc pressed");
            TERMINATE.store(true, Ordering::SeqCst);
        })
        .expect("Error setting Ctrl-C handler");
    }
    #[pyfn(m)]
    fn setup_logging(_py: Python, default_log_level: String) {
        env_logger::Builder::from_env(Env::new().filter_or("FLATTERER_LOG", &default_log_level))
            .format_timestamp_millis()
            .format_target(false)
            .init();
    }
    #[pyfn(m)]
    fn flatten_rs(
        _py: Python,
        input_file: String,
        output_dir: String,
        csv: bool,
        xlsx: bool,
        sqlite: bool,
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
        sqlite_path: String,
        preview: usize,
        log_error: bool,
    ) -> Result<()> {
        let flat_files_res = FlatFiles::new(
            output_dir,
            csv,
            xlsx,
            sqlite,
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
            selectors.push(path.to_string());
        }

        if let Err(err) = flat_files_res {
            if log_error {
                log::error!("{}", err)
            };
            return Err(err.into());
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if !fields.is_empty() {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                if log_error {
                    log::error!("{}", err)
                };
                return Err(err.into());
            }
        }
        if !tables.is_empty() {
            if let Err(err) = flat_files.use_tables_csv(tables, only_tables) {
                if log_error {
                    log::error!("{}", err)
                };
                return Err(err.into());
            }
        }

        if !sqlite_path.is_empty() {
            flat_files.sqlite_path = sqlite_path
        }

        if preview > 0 {
            flat_files.preview = preview
        }

        let file;

        match File::open(&input_file) {
            Ok(input) => {
                file = BufReader::new(input);
            }
            Err(err) => {
                if log_error {
                    log::error!("Can not open file `{}`: {}", input_file, &err)
                };
                let result: Result<()> = Err(err.into());
                return result.wrap_err_with(|| format!("Can not open file `{}`", input_file));
            }
        };

        if let Err(err) = flatten(file, flat_files, selectors, json_lines) {
            if log_error {
                log::error!("{}", err)
            };
            return Err(err.into());
        };

        log::info!("All finished with no errors!");
        Ok(())
    }

    #[pyfn(m)]
    fn iterator_flatten_rs(
        py: Python,
        mut objs: &PyIterator,
        output_dir: String,
        csv: bool,
        xlsx: bool,
        sqlite: bool,
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
        sqlite_path: String,
        preview: usize,
        log_error: bool,
    ) -> Result<()> {
        let flat_files_res = FlatFiles::new(
            output_dir,
            csv,
            xlsx,
            sqlite,
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
            if log_error {
                log::error!("{}", err)
            };
            return Err(err.into());
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if !fields.is_empty() {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                if log_error {
                    log::error!("{}", err)
                };
                return Err(err.into());
            }
        }

        if !tables.is_empty() {
            if let Err(err) = flat_files.use_tables_csv(tables, only_tables) {
                if log_error {
                    log::error!("{}", err)
                };
                return Err(err.into());
            }
        }

        if !sqlite_path.is_empty() {
            flat_files.sqlite_path = sqlite_path
        }

        if preview > 0 {
            flat_files.preview = preview
        }

        let (sender, receiver) = bounded(1000);

        let handler = thread::spawn(move || -> Result<()> {
            for value in receiver {
                flat_files.process_value(value, vec![]);
                flat_files.create_rows()?;
            }

            flat_files.write_files()?;
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
                        if log_error {
                            log::error!("{}", err)
                        };
                        return Err(err.into());
                    }
                }
                Err(err) => {
                    if log_error {
                        log::error!("{}", err)
                    };
                    return Err(err.into());
                }
            }

            drop(gilpool)
        }

        drop(sender);

        match handler.join() {
            Ok(result) => {
                if let Err(err) = result {
                    if log_error {
                        log::error!("{}", &err)
                    };
                    return Err(err.into());
                }
            }
            Err(err) => {
                if log_error {
                    log::error!("{:?}", &err)
                };
                return Err(eyre::eyre!("{:?}", &err));
            }
        }
        Ok(())
    }

    log::info!("All finished with no errors!");
    Ok(())
}
