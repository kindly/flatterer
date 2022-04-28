use crossbeam_channel::{bounded, Sender, Receiver};
use datapackage_convert::{merge_datapackage_with_options, datapackage_to_parquet_with_options, datapackage_to_sqlite_with_options};
use eyre::{Result, WrapErr, eyre};
use libflatterer::{flatten, Options, TERMINATE, FlatFiles};
use serde_json::Value;
use std::thread;
use std::path::PathBuf;

use env_logger::Env;
use pyo3::prelude::*;
use pyo3::types::PyIterator;
use std::fs::{File, remove_dir_all};
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
        parquet: bool,
        main_table_name: String,
        tables_csv: String,
        only_tables: bool,
        fields_csv: String,
        only_fields: bool,
        inline_one_to_one: bool,
        path_separator: String,
        preview: usize,
        table_prefix: String,
        id_prefix: String,
        emit_obj: Vec<Vec<String>>,
        force: bool,
        schema: String,
        schema_titles: String,
        path: Vec<String>,
        json_stream: bool,
        ndjson: bool,
        sqlite_path: String,
        threads: usize,
        log_error: bool,
    ) -> Result<()> {

        let mut op = Options::default();

        op.csv = csv;
        op.xlsx = xlsx;
        op.sqlite = sqlite;
        op.parquet = parquet;
        op.main_table_name = main_table_name;
        op.tables_csv = tables_csv;
        op.only_tables = only_tables;
        op.fields_csv = fields_csv;
        op.only_fields = only_fields;
        op.inline_one_to_one = inline_one_to_one;
        op.path_separator = path_separator;
        op.preview = preview;
        op.table_prefix = table_prefix;
        op.id_prefix = id_prefix;
        op.emit_obj = emit_obj;
        op.force = force;
        op.schema = schema;
        op.schema_titles = schema_titles;
        op.path = path;
        op.json_stream = json_stream;
        op.ndjson = ndjson;
        op.sqlite_path = sqlite_path;
        op.threads = threads;

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

        if let Err(err) = flatten(file, output_dir, op) {
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
        parquet: bool,
        main_table_name: String,
        tables_csv: String,
        only_tables: bool,
        fields_csv: String,
        only_fields: bool,
        inline_one_to_one: bool,
        path_separator: String,
        preview: usize,
        table_prefix: String,
        id_prefix: String,
        emit_obj: Vec<Vec<String>>,
        force: bool,
        schema: String,
        schema_titles: String,
        sqlite_path: String,
        threads: usize,
        log_error: bool,
    ) -> Result<()> {
        let mut options = Options::default();

        options.csv = csv;
        options.xlsx = xlsx;
        options.sqlite = sqlite;
        options.parquet = parquet;
        options.main_table_name = main_table_name;
        options.tables_csv = tables_csv;
        options.only_tables = only_tables;
        options.fields_csv = fields_csv;
        options.only_fields = only_fields;
        options.inline_one_to_one = inline_one_to_one;
        options.path_separator = path_separator;
        options.preview = preview;
        options.table_prefix = table_prefix;
        options.id_prefix = id_prefix;
        options.emit_obj = emit_obj;
        options.force = force;
        options.schema = schema;
        options.schema_titles = schema_titles;
        options.sqlite_path = sqlite_path;
        options.threads = threads;


        let final_output_path = PathBuf::from(output_dir);
        let parts_path = final_output_path.join("parts");

        if threads == 0 {
            options.threads = num_cpus::get();
        }

        if options.threads > 1 {
            if options.xlsx {
                log::warn!("XLSX output not supported in multi threaded mode");
                options.xlsx = false;
            }
            if final_output_path.is_dir() {
                if options.force {
                    remove_dir_all(&final_output_path)?;
                } else {
                    return Err(eyre!("Output directory {} already exists", final_output_path.to_string_lossy()));
                }
            }

            std::fs::create_dir_all(&parts_path)?        
        }

        let (sender, initial_receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(1000);

        let mut output_paths = vec![];
        let mut handlers = vec![];

        for index in 0..options.threads {
            let mut options_clone = options.clone();

            let mut output_path = final_output_path.clone();

            if options.threads > 1 {
                options_clone.id_prefix = format!("{}.{}", index, options_clone.id_prefix);
                options_clone.csv = true;
                options_clone.sqlite = false;
                options_clone.parquet = false;
                output_path = parts_path.join(index.to_string());
            }
            output_paths.push(output_path.clone().to_string_lossy().to_string());

            let mut flat_files = FlatFiles::new(
                output_path.clone().to_string_lossy().to_string(),
                options_clone.clone(),
            )?;

            let receiver = initial_receiver.clone();

            let handler = thread::spawn(move || -> Result<()> {
                for json_bytes in receiver {
                    match serde_json::from_slice::<Value>(&json_bytes) {
                        Ok(value) => {
                            flat_files.process_value(value, vec![]);
                            flat_files.create_rows()?;
                        }
                        Err(err) => {
                            if log_error {
                                log::error!("{}", err)
                            };
                            return Err(err.into());
                        }
                    }
                }

                flat_files.write_files()?;
                Ok(())
            });
            handlers.push(handler)
        }

        drop(initial_receiver);

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

            let json_bytes = PyAny::extract::<&[u8]>(result?)?.to_owned();

            if let Err(err) = sender.send(json_bytes) {
                if log_error {
                    log::error!("{}", err)
                };
                return Err(err.into());
            }


            drop(gilpool)
        }

        drop(sender);

        for handler in handlers {
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
        }

        if options.threads > 1 {
            let op = datapackage_convert::Options::builder()
                .delete_input_csv(true)
                .build();
            merge_datapackage_with_options(final_output_path.clone(), output_paths, op)?;

            remove_dir_all(&parts_path)?;

            if options.parquet {
                let op = datapackage_convert::Options::builder().build();
                datapackage_to_parquet_with_options(
                    final_output_path.join("parquet"),
                    final_output_path.to_string_lossy().into(),
                    op,
                )?
            }

            if options.sqlite {
                let op = datapackage_convert::Options::builder().build();
                if options.sqlite_path.is_empty() {
                    options.sqlite_path = final_output_path.join("sqlite.db").to_string_lossy().into();
                }
                datapackage_to_sqlite_with_options(
                    options.sqlite_path,
                    final_output_path.to_string_lossy().into(),
                    op,
                )?
            }

            if !options.csv {
                remove_dir_all(final_output_path.join("csv"))?        }

            libflatterer::write_metadata_csvs_from_datapackage(final_output_path)?;
        }

        Ok(())
    }

    log::info!("All finished with no errors!");
    Ok(())
}
