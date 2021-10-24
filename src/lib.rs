use std::collections::HashMap;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::io::{self, BufReader, Error as IoError, ErrorKind, Read, Write};
use std::path::PathBuf;
use std::{panic, thread};

use crossbeam_channel::{bounded, Receiver, Sender};
use csv::{ByteRecord, Reader, ReaderBuilder, Writer, WriterBuilder};
use itertools::{izip, Itertools};
use pyo3::prelude::*;
use pyo3::types::PyIterator;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Deserializer, Map, Value};
use smallvec::{smallvec, SmallVec};
use xlsxwriter::Workbook;
use yajlish::ndjson_handler::{NdJsonHandler, Selector};
use yajlish::Parser;

#[pymodule]
fn flatterer(_py: Python, m: &PyModule) -> PyResult<()> {
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
        inline_one_to_one: bool,
    ) -> PyResult<()> {
        let flat_files_res = FlatFiles::new(
            output_dir.to_string(),
            csv,
            xlsx,
            force,
            main_table_name,
            emit_path,
            inline_one_to_one,
        );

        let mut selectors = vec![];

        if path != "" {
            selectors.push(Selector::Identifier(format!("\"{}\"", path.to_string())));
        }

        if flat_files_res.is_err() {
            let err = flat_files_res.unwrap_err();
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "{:?}",
                err
            )));
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if fields != "" {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
            }
        }

        let file;

        match File::open(input_file) {
            Ok(input) => {
                file = BufReader::new(input);
            }
            Err(err) => {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
            }
        };

        if json_lines {
            if let Err(err) = flatten_from_jl(file, flat_files) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
            }
        } else {
            if let Err(err) = flatten(file, flat_files, selectors) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
            }
        }

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
        inline_one_to_one: bool,
    ) -> PyResult<()> {
        let flat_files_res = FlatFiles::new(
            output_dir.to_string(),
            csv,
            xlsx,
            force,
            main_table_name,
            emit_path,
            inline_one_to_one,
        );

        if flat_files_res.is_err() {
            let err = flat_files_res.unwrap_err();
            return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                "{:?}",
                err
            )));
        }

        let mut flat_files = flat_files_res.unwrap(); //already checked error

        if fields != "" {
            if let Err(err) = flat_files.use_fields_csv(fields, only_fields) {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
            }
        }

        let (sender, receiver) = bounded(1000);

        let handler = thread::spawn(move || -> PyResult<()> {
            for value in receiver {
                flat_files.process_value(value);
                if let Err(err) = flat_files.create_rows() {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "{:?}",
                        err
                    )));
                }
            }

            if let Err(err) = flat_files.write_files() {
                return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                    "{:?}",
                    err
                )));
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

            match serde_json::from_slice::<Value>(&json_bytes) {
                Ok(value) => {
                    if let Err(err) = sender.send(value) {
                        return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                            "{:?}",
                            err
                        )));
                    }
                }
                Err(err) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "{:?}",
                        err
                    )))
                }
            }

            drop(gilpool)
        }

        drop(sender);

        match handler.join() {
            Ok(result) => {
                if let Err(err) = result {
                    return Err(PyErr::new::<pyo3::exceptions::PyIOError, _>(format!(
                        "{:?}",
                        err
                    )));
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

#[derive(Clone, Debug)]
pub enum PathItem {
    Key(String),
    Index(usize),
}

impl fmt::Display for PathItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PathItem::Key(key) => write!(f, "{}", key),
            PathItem::Index(index) => write!(f, "{}", index),
        }
    }
}

#[derive(Debug)]
pub struct FlatFiles {
    output_path: PathBuf,
    csv: bool,
    xlsx: bool,
    main_table_name: String,
    emit_obj: Vec<Vec<String>>,
    row_number: u128,
    date_regexp: Regex,
    table_rows: HashMap<String, Vec<Map<String, Value>>>,
    tmp_csvs: HashMap<String, csv::Writer<File>>,
    table_metadata: HashMap<String, TableMetadata>,
    only_fields: bool,
    inline_one_to_one: bool,
    one_to_many_arrays: Vec<Vec<String>>,
    one_to_one_arrays: Vec<Vec<String>>,
}

#[derive(Serialize, Debug)]
pub struct TableMetadata {
    field_type: Vec<String>,
    fields: Vec<String>,
    field_counts: Vec<u32>,
    rows: u32,
    ignore: bool,
    ignore_fields: Vec<bool>,
}

#[derive(Debug, Deserialize)]
struct FieldsRecord {
    table_name: String,
    field_name: String,
    field_type: String,
}

struct JLWriter {
    pub buf: Vec<u8>,
    pub buf_sender: Sender<Vec<u8>>,
}

impl Write for JLWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf == [b'\n'] {
            self.buf_sender.send(self.buf.clone()).unwrap();
            self.buf.clear();
            Ok(buf.len())
        } else {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl FlatFiles {
    pub fn new(
        output_dir: String,
        csv: bool,
        xlsx: bool,
        force: bool,
        main_table_name: String,
        emit_obj: Vec<Vec<String>>,
        inline_one_to_one: bool,
    ) -> Result<FlatFiles, IoError> {
        let output_path = PathBuf::from(output_dir.clone());
        if output_path.is_dir() {
            if force {
                remove_dir_all(&output_path)?;
            } else {
                return Err(IoError::new(
                    ErrorKind::AlreadyExists,
                    format!("Directory {} already exists", output_dir),
                ));
            }
        }
        if csv {
            let csv_path = output_path.join("csv");
            create_dir_all(&csv_path)?;
        }

        let tmp_path = output_path.join("tmp");
        create_dir_all(&tmp_path)?;

        Ok(FlatFiles {
            output_path,
            csv,
            xlsx,
            main_table_name,
            emit_obj,
            row_number: 1,
            date_regexp: Regex::new(r"^([1-3]\d{3})-(\d{2})-(\d{2})([T ](\d{2}):(\d{2}):(\d{2}(?:\.\d*)?)((-(\d{2}):(\d{2})|Z)?))?$").unwrap(),
            table_rows: HashMap::new(),
            tmp_csvs: HashMap::new(),
            table_metadata: HashMap::new(),
            only_fields: false,
            inline_one_to_one,
            one_to_many_arrays: Vec::new(),
            one_to_one_arrays: Vec::new(),
        })
    }

    fn handle_obj(
        &mut self,
        mut obj: Map<String, Value>,
        emit: bool,
        full_path: Vec<PathItem>,
        no_index_path: Vec<String>,
        one_to_many_full_paths: Vec<Vec<PathItem>>,
        one_to_many_no_index_paths: Vec<Vec<String>>,
        one_to_one_key: bool,
    ) -> Option<Map<String, Value>> {
        let mut to_insert: SmallVec<[(String, Value); 30]> = smallvec![];
        let mut to_delete: SmallVec<[String; 30]> = smallvec![];

        let mut one_to_one_array: SmallVec<[(String, Value); 30]> = smallvec![];

        for (key, value) in obj.iter_mut() {
            if let Some(arr) = value.as_array() {
                let mut str_count = 0;
                let mut obj_count = 0;
                let arr_length = arr.len();
                for array_value in arr {
                    if array_value.is_object() {
                        obj_count += 1
                    };
                    if array_value.is_string() {
                        str_count += 1
                    };
                }
                if arr_length == 0 {
                    to_delete.push(key.clone());
                }
                if str_count == arr_length {
                    let keys: Vec<String> = arr
                        .iter()
                        .map(|val| (val.as_str().unwrap().to_string())) //value known as str
                        .collect();
                    let new_value = json!(keys.join(","));
                    to_insert.push((key.clone(), new_value))
                } else if obj_count == arr_length {
                    to_delete.push(key.clone());
                    let mut removed_array = value.take(); // obj.remove(&key).unwrap(); //key known
                    let my_array = removed_array.as_array_mut().unwrap(); //key known as array
                    for (i, array_value) in my_array.iter_mut().enumerate() {
                        let my_value = array_value.take();

                        let mut new_full_path = full_path.clone();
                        new_full_path.push(PathItem::Key(key.clone()));
                        new_full_path.push(PathItem::Index(i));

                        let mut new_one_to_many_full_paths = one_to_many_full_paths.clone();
                        new_one_to_many_full_paths.push(new_full_path.clone());

                        let mut new_no_index_path = no_index_path.clone();
                        new_no_index_path.push(key.clone());

                        let mut new_one_to_many_no_index_paths = one_to_many_no_index_paths.clone();
                        new_one_to_many_no_index_paths.push(new_no_index_path.clone());

                        if self.inline_one_to_one
                            && !self.one_to_many_arrays.contains(&new_no_index_path)
                        {
                            if arr_length == 1 {
                                one_to_one_array.push((key.clone(), my_value.clone()));
                                if !self.one_to_one_arrays.contains(&new_no_index_path) {
                                    self.one_to_one_arrays.push(new_no_index_path.clone())
                                }
                            } else if arr_length > 1 {
                                self.one_to_one_arrays.retain(|x| x != &new_no_index_path);
                                self.one_to_many_arrays.push(new_no_index_path.clone())
                            }
                        }

                        if let Value::Object(my_obj) = my_value {
                            if !one_to_one_key {
                                self.handle_obj(
                                    my_obj,
                                    true,
                                    new_full_path,
                                    new_no_index_path,
                                    new_one_to_many_full_paths,
                                    new_one_to_many_no_index_paths,
                                    false,
                                );
                            }
                        }
                    }
                } else {
                    let json_value = json!(format!("{}", value));
                    to_insert.push((key.clone(), json_value));
                }
            }
        }

        let mut one_to_one_array_keys = vec![];

        for (key, value) in one_to_one_array {
            one_to_one_array_keys.push(key.clone());
            obj.insert(key, value);
        }

        for (key, value) in obj.iter_mut() {
            if value.is_object() {
                let my_value = value.take();
                to_delete.push(key.clone());

                let mut new_full_path = full_path.clone();
                new_full_path.push(PathItem::Key(key.clone()));
                let mut new_no_index_path = no_index_path.clone();
                new_no_index_path.push(key.clone());

                let mut emit_child = false;
                if self
                    .emit_obj
                    .iter()
                    .any(|emit_path| emit_path == &new_no_index_path)
                {
                    emit_child = true;
                }
                if let Value::Object(my_value) = my_value {
                    let new_obj = self.handle_obj(
                        my_value,
                        emit_child,
                        new_full_path,
                        new_no_index_path,
                        one_to_many_full_paths.clone(),
                        one_to_many_no_index_paths.clone(),
                        one_to_one_array_keys.contains(&key),
                    );
                    if let Some(mut my_obj) = new_obj {
                        for (new_key, new_value) in my_obj.iter_mut() {
                            let mut object_key = String::with_capacity(100);
                            object_key.push_str(key);
                            object_key.push_str("_");
                            object_key.push_str(new_key);

                            to_insert.push((object_key, new_value.take()));
                        }
                    }
                }
            }
        }
        for key in to_delete {
            obj.remove(&key);
        }
        for (key, value) in to_insert {
            obj.insert(key, value);
        }

        if emit {
            self.process_obj(
                obj,
                no_index_path,
                one_to_many_full_paths,
                one_to_many_no_index_paths,
            );
            None
        } else {
            Some(obj)
        }
    }

    pub fn process_obj(
        &mut self,
        mut obj: Map<String, Value>,
        no_index_path: Vec<String>,
        one_to_many_full_paths: Vec<Vec<PathItem>>,
        one_to_many_no_index_paths: Vec<Vec<String>>,
    ) {
        let mut path_iter = one_to_many_full_paths
            .iter()
            .zip(one_to_many_no_index_paths)
            .peekable();

        if one_to_many_full_paths.len() == 0 {
            obj.insert(
                String::from("_link"),
                Value::String(self.row_number.to_string()),
            );
        }

        while let Some((full, no_index)) = path_iter.next() {
            if path_iter.peek().is_some() {
                obj.insert(
                    ["_link_".to_string(), no_index.iter().join("_")].concat(),
                    Value::String(
                        [
                            self.row_number.to_string(),
                            ".".to_string(),
                            full.iter().join("."),
                        ]
                        .concat(),
                    ),
                );
            } else {
                obj.insert(
                    String::from("_link"),
                    Value::String(
                        [
                            self.row_number.to_string(),
                            ".".to_string(),
                            full.iter().join("."),
                        ]
                        .concat(),
                    ),
                );
            }
        }

        obj.insert(
            ["_link_", &self.main_table_name].concat(),
            Value::String(self.row_number.to_string()),
        );

        let mut table_name = no_index_path.join("_");

        if table_name == "" {
            table_name = self.main_table_name.clone();
        }

        if !self.table_rows.contains_key(&table_name) {
            self.table_rows.insert(table_name, vec![obj]);
        } else {
            let current_list = self.table_rows.get_mut(&table_name).unwrap(); //key known
            current_list.push(obj)
        }
    }

    pub fn create_rows(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        for (table, rows) in self.table_rows.iter_mut() {
            if !self.tmp_csvs.contains_key(table) {
                self.tmp_csvs.insert(
                    table.clone(),
                    WriterBuilder::new()
                        .flexible(true)
                        .from_path(self.output_path.join(format!("tmp/{}.csv", table)))?,
                );
            }

            if !self.table_metadata.contains_key(table) {
                self.table_metadata.insert(
                    table.clone(),
                    TableMetadata {
                        fields: vec![],
                        field_counts: vec![],
                        field_type: vec![],
                        rows: 0,
                        ignore: false,
                        ignore_fields: vec![],
                    },
                );
            }

            let table_metadata = self.table_metadata.get_mut(table).unwrap(); //key known
            let writer = self.tmp_csvs.get_mut(table).unwrap(); //key known

            for row in rows {
                let mut output_row: SmallVec<[String; 30]> = smallvec![];
                for (num, field) in table_metadata.fields.iter().enumerate() {
                    if let Some(value) = row.get_mut(field) {
                        table_metadata.field_counts[num] += 1;
                        output_row.push(value_convert(
                            value.take(),
                            &mut table_metadata.field_type,
                            num,
                            &self.date_regexp,
                        ));
                    } else {
                        output_row.push("".to_string());
                    }
                }
                for (key, value) in row {
                    if !table_metadata.fields.contains(key) && !self.only_fields {
                        table_metadata.fields.push(key.clone());
                        table_metadata.field_counts.push(1);
                        table_metadata.field_type.push("".to_string());
                        table_metadata.ignore_fields.push(false);
                        output_row.push(value_convert(
                            value.take(),
                            &mut table_metadata.field_type,
                            table_metadata.fields.len() - 1,
                            &self.date_regexp,
                        ));
                    }
                }
                if output_row.len() > 0 {
                    table_metadata.rows += 1;
                    writer.write_record(&output_row)?;
                }
            }
        }
        for val in self.table_rows.values_mut() {
            val.clear();
        }
        Ok(())
    }

    pub fn process_value(&mut self, value: Value) {
        if let Value::Object(obj) = value {
            self.handle_obj(obj, true, vec![], vec![], vec![], vec![], false);
            self.row_number += 1;
        }
    }

    pub fn use_fields_csv(
        &mut self,
        filepath: String,
        only_fields: bool,
    ) -> Result<(), csv::Error> {
        let mut fields_reader = Reader::from_path(filepath)?;

        if only_fields {
            self.only_fields = true;
        }

        for row in fields_reader.deserialize() {
            let row: FieldsRecord = row?;

            if !self.table_metadata.contains_key(&row.table_name) {
                self.table_metadata.insert(
                    row.table_name.clone(),
                    TableMetadata {
                        fields: vec![],
                        field_counts: vec![],
                        field_type: vec![],
                        rows: 0,
                        ignore: false,
                        ignore_fields: vec![],
                    },
                );
            }
            let table_metadata = self.table_metadata.get_mut(&row.table_name).unwrap(); //key known
            table_metadata.fields.push(row.field_name);
            table_metadata.field_counts.push(0);
            table_metadata.field_type.push(row.field_type);
            table_metadata.ignore_fields.push(false);
        }

        return Ok(());
    }

    pub fn mark_ignore(&mut self) {
        let one_to_many_table_names = self
            .one_to_many_arrays
            .iter()
            .map(|item| item.join("_"))
            .collect_vec();

        for (table_name, metadata) in self.table_metadata.iter_mut() {
            let table_name_with_underscore = if table_name == &self.main_table_name {
                "".to_string()
            } else {
                format!("{}_", table_name)
            };

            for (num, field) in metadata.fields.iter().enumerate() {
                let full_path = format!("{}{}", table_name_with_underscore, field);
                for one_to_many_table_name in &one_to_many_table_names {
                    if full_path.starts_with(one_to_many_table_name)
                        && !table_name_with_underscore.starts_with(one_to_many_table_name)
                    {
                        metadata.ignore_fields[num] = true;
                    }
                }
            }
        }

        for table_path in &self.one_to_one_arrays {
            let table_name = table_path.iter().join("_");
            if let Some(table_metadata) = self.table_metadata.get_mut(&table_name) {
                table_metadata.ignore = true
            }
        }
    }

    pub fn write_files(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        self.mark_ignore();

        for tmp_csv in self.tmp_csvs.values_mut() {
            tmp_csv.flush()?;
        }

        if self.csv {
            self.write_csvs()?;
        };

        if self.xlsx {
            self.write_xlsx()?;
        };

        let tmp_path = self.output_path.join("tmp");
        remove_dir_all(&tmp_path)?;

        let metadata_file = File::create(self.output_path.join("data_package.json"))?;

        let mut resources = vec![];

        for table_name in self.table_metadata.keys().sorted() {
            let metadata = self.table_metadata.get(table_name).unwrap();
            let mut fields = vec![];
            if metadata.rows == 0 || metadata.ignore {
                continue;
            }

            for (field, field_type, count, ignore) in izip!(
                &metadata.fields,
                &metadata.field_type,
                &metadata.field_counts,
                &metadata.ignore_fields
            ) {
                let field = json!({
                    "name": field,
                    "type": field_type,
                    "count": count,
                });
                if !ignore {
                    fields.push(field);
                }
            }

            let mut resource = json!({
                "profile": "tabular-data-resource",
                "name": table_name,
                "schema": {
                    "fields": fields,
                    "primaryKey": "_link"
                }
            });
            if self.csv {
                resource.as_object_mut().unwrap().insert(
                    "path".to_string(),
                    Value::String(format!("csv/{}.csv", table_name)),
                );
            }

            resources.push(resource)
        }

        let mut fields_writer = Writer::from_path(self.output_path.join("fields.csv"))?;

        fields_writer.write_record(["table_name", "field_name", "field_type", "count"])?;
        for table_name in self.table_metadata.keys().sorted() {
            let metadata = self.table_metadata.get(table_name).unwrap();
            if metadata.rows == 0 || metadata.ignore {
                continue;
            }
            for (field, field_type, count, ignore) in izip!(
                &metadata.fields,
                &metadata.field_type,
                &metadata.field_counts,
                &metadata.ignore_fields
            ) {
                if !ignore {
                    fields_writer.write_record([
                        table_name,
                        field,
                        field_type,
                        &count.to_string(),
                    ])?;
                }
            }
        }

        let data_package = json!({
            "profile": "tabular-data-package",
            "resources": resources
        });

        serde_json::to_writer_pretty(metadata_file, &data_package)?;

        Ok(())
    }

    pub fn write_csvs(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let tmp_path = self.output_path.join("tmp");
        let csv_path = self.output_path.join("csv");

        for table_name in self.tmp_csvs.keys() {
            let metadata = self.table_metadata.get(table_name).unwrap(); //key known
            if metadata.rows == 0 || metadata.ignore {
                continue;
            }

            let csv_reader = ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .from_path(tmp_path.join(format!("{}.csv", table_name)))?;

            let mut csv_writer =
                WriterBuilder::new().from_path(csv_path.join(format!("{}.csv", table_name)))?;

            let field_count = &metadata.fields.len();

            let mut non_ignored_fields = vec![];

            for (field, ignore_field) in izip!(&metadata.fields, &metadata.ignore_fields) {
                if !ignore_field {
                    non_ignored_fields.push(field)
                }
            }

            csv_writer.write_record(&non_ignored_fields)?;

            let mut output_row = ByteRecord::new();

            for row in csv_reader.into_byte_records() {
                let mut this_row = row?;

                while &this_row.len() != field_count {
                    this_row.push_field(b"");
                }

                for (data, ignore_field) in izip!(this_row.iter(), &metadata.ignore_fields) {
                    if !ignore_field {
                        output_row.push_field(data);
                    }
                }

                csv_writer.write_byte_record(&output_row)?;
                output_row.clear();
            }
        }

        Ok(())
    }

    pub fn write_xlsx(&mut self) -> Result<(), Box<dyn Error + Sync + Send>> {
        let tmp_path = self.output_path.join("tmp");

        let workbook = Workbook::new_with_options(
            &self.output_path.join("output.xlsx").to_string_lossy(),
            true,
            Some(&tmp_path.to_string_lossy()),
            false,
        );

        for table_name in self.tmp_csvs.keys() {
            let metadata = self.table_metadata.get(table_name).unwrap(); //key known
            if metadata.rows == 0 || metadata.ignore {
                continue;
            }
            let mut new_table_name = table_name.clone();
            new_table_name.truncate(31);
            let mut worksheet = workbook.add_worksheet(Some(&new_table_name))?;

            let csv_reader = ReaderBuilder::new()
                .has_headers(false)
                .flexible(true)
                .from_path(tmp_path.join(format!("{}.csv", table_name)))?;

            let mut col_index = 0;

            for (field, ignore_field) in izip!(&metadata.fields, &metadata.ignore_fields) {
                if !ignore_field {
                    worksheet.write_string(0, col_index, &field, None)?;
                    col_index += 1;
                }
            }

            for (row_num, row) in csv_reader.into_records().enumerate() {
                col_index = 0;
                let this_row = row?;
                for (cell, ignore) in izip!(this_row.iter(), &metadata.ignore_fields) {
                    if *ignore {
                        continue;
                    }
                    if metadata.field_type[usize::from(col_index)] == "number" {
                        if let Ok(number) = cell.parse::<f64>() {
                            worksheet.write_number(
                                (row_num + 1).try_into()?,
                                col_index,
                                number,
                                None,
                            )?;
                        } else {
                            worksheet.write_string(
                                (row_num + 1).try_into()?,
                                col_index,
                                cell,
                                None,
                            )?;
                        };
                    } else {
                        worksheet.write_string((row_num + 1).try_into()?, col_index, cell, None)?;
                    }
                    col_index += 1
                }
            }
        }
        workbook.close()?;

        return Ok(());
    }
}

fn value_convert(
    value: Value,
    field_type: &mut Vec<String>,
    num: usize,
    date_re: &Regex,
) -> String {
    //let value_type = output_fields.get("type");
    let value_type = &field_type[num];

    match value {
        Value::String(val) => {
            if value_type != &"text".to_string() {
                if date_re.is_match(&val) {
                    field_type[num] = "date".to_string();
                } else {
                    field_type[num] = "text".to_string();
                }
            }
            val
        }
        Value::Null => {
            if value_type == &"".to_string() {
                field_type[num] = "null".to_string();
            }
            "".to_string()
        }
        Value::Number(number) => {
            if value_type != &"text".to_string() {
                field_type[num] = "number".to_string();
            }
            number.to_string()
        }
        Value::Bool(bool) => {
            if value_type != &"text".to_string() {
                field_type[num] = "boolean".to_string();
            }
            bool.to_string()
        }
        Value::Array(_) => {
            if value_type != &"text".to_string() {
                field_type[num] = "text".to_string();
            }
            format!("{}", value)
        }
        Value::Object(_) => {
            if value_type != &"text".to_string() {
                field_type[num] = "text".to_string();
            }
            format!("{}", value)
        }
    }
}

pub fn flatten_from_jl<R: Read>(input: R, mut flat_files: FlatFiles) -> Result<(), Box<dyn Error>> {
    let (value_sender, value_receiver) = bounded(1000);

    let thread = thread::spawn(move || -> Result<(), Box<dyn Error + Sync + Send>> {
        for value in value_receiver {
            flat_files.process_value(value);
            flat_files.create_rows()?
        }

        flat_files.write_files()?;
        Ok(())
    });

    let stream = Deserializer::from_reader(input).into_iter::<Value>();
    for value_result in stream {
        let value = value_result?;
        value_sender.send(value)?;
    }
    drop(value_sender);

    match thread.join() {
        Ok(result) => {
            if let Err(err) = result {
                return Err(err);
            }
        }
        Err(err) => panic::resume_unwind(err),
    }

    Ok(())
}

pub fn flatten<R: Read>(
    mut input: BufReader<R>,
    mut flat_files: FlatFiles,
    selectors: Vec<Selector>,
) -> Result<(), Box<dyn Error>> {
    let (buf_sender, buf_receiver): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = bounded(1000);

    let thread = thread::spawn(move || -> Result<(), Box<dyn Error + Sync + Send>> {
        for buf in buf_receiver.iter() {
            let value = serde_json::from_slice::<Value>(&buf)?;
            flat_files.process_value(value);
            flat_files.create_rows()?;
        }

        flat_files.write_files()?;
        Ok(())
    });

    let mut jl_writer = JLWriter {
        buf: vec![],
        buf_sender,
    };

    let mut handler = NdJsonHandler::new(&mut jl_writer, selectors);
    let mut parser = Parser::new(&mut handler);

    parser.parse(&mut input)?;

    drop(jl_writer);

    match thread.join() {
        Ok(result) => {
            if let Err(err) = result {
                return Err(err);
            }
        }
        Err(err) => panic::resume_unwind(err),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::read_to_string;

    #[test]
    fn full_test() {
        let tmp_dir = TempDir::new().unwrap();
        let output_dir = tmp_dir.path().join("output");
        let output_path = output_dir.to_string_lossy().into_owned();
        let flat_files = FlatFiles::new(
            output_path.clone(),
            true,
            true,
            true,
            "main".to_string(),
            vec![],
            false,
        )
        .unwrap();
        flatten(
            BufReader::new(File::open("fixtures/basic.json").unwrap()),
            flat_files,
            vec![]
        ).unwrap();

        for test_file in ["data_package.json", "fields.csv", "csv/main.csv", "csv/platforms.csv"] {
            let expected = read_to_string(format!("fixtures/expected_basic/{}", test_file)).unwrap();
            println!("{}", expected);
            let output = read_to_string(format!("{}/{}", output_path.clone(), test_file)).unwrap();
            println!("{}", output);
            assert!(expected == output);
        }
    }

    #[test]
    fn full_test_inline() {
        let tmp_dir = TempDir::new().unwrap();
        let output_dir = tmp_dir.path().join("output");
        let output_path = output_dir.to_string_lossy().into_owned();
        let flat_files = FlatFiles::new(
            output_path.clone(),
            true,
            true,
            true,
            "main".to_string(),
            vec![],
            true,
        )
        .unwrap();
        flatten(
            BufReader::new(File::open("fixtures/basic.json").unwrap()),
            flat_files,
            vec![]
        ).unwrap();

        for test_file in ["data_package.json", "fields.csv", "csv/main.csv", "csv/platforms.csv"] {
            let expected = read_to_string(format!("fixtures/expected_basic_inline/{}", test_file)).unwrap();
            println!("{}", expected);
            let output = read_to_string(format!("{}/{}", output_path.clone(), test_file)).unwrap();
            println!("{}", output);
            assert!(expected == output);
        }
    }

    #[test]
    fn check_nesting() {
        let myjson = json!({
            "a": "a",
            "c": ["a", "b", "c"],
            "d": {"da": "da", "db": "2005-01-01"},
            "e": [{"ea": 1, "eb": "eb2"},
                  {"ea": 2, "eb": "eb2"}],
        });

        let tmp_dir = TempDir::new().unwrap();

        let mut flat_files = FlatFiles::new(
            tmp_dir.path().join("output").to_string_lossy().into_owned(),
            true,
            true,
            true,
            "main".to_string(),
            vec![],
            false,
        )
        .unwrap();

        flat_files.process_value(myjson.clone());

        let expected_table_rows = json!({
          "e": [
            {
              "_link": "1.e.0",
              "_link_main": "1",
              "ea": 1,
              "eb": "eb2"
            },
            {
              "_link": "1.e.1",
              "_link_main": "1",
              "ea": 2,
              "eb": "eb2"
            }
          ],
          "main": [
            {
              "_link": "1",
              "_link_main": "1",
              "a": "a",
              "c": "a,b,c",
              "d_da": "da",
              "d_db": "2005-01-01"
            }
          ]
        });

        //println!("{}", serde_json::to_value(&flat_files.table_rows).unwrap());

        assert!(expected_table_rows == serde_json::to_value(&flat_files.table_rows).unwrap());

        flat_files.create_rows().unwrap();

        let expected_metadata = json!({
          "e": {
            "field_type": [
              "text",
              "text",
              "number",
              "text"
            ],
            "fields": [
              "_link",
              "_link_main",
              "ea",
              "eb"
            ],
            "field_counts": [
              2,
              2,
              2,
              2
            ],
            "rows": 2,
            "ignore": false,
            "ignore_fields": [
              false,
              false,
              false,
              false
            ]
          },
          "main": {
            "field_type": [
              "text",
              "text",
              "text",
              "text",
              "text",
              "date"
            ],
            "fields": [
              "_link",
              "_link_main",
              "a",
              "c",
              "d_da",
              "d_db"
            ],
            "field_counts": [
              1,
              1,
              1,
              1,
              1,
              1
            ],
            "rows": 1,
            "ignore": false,
            "ignore_fields": [
              false,
              false,
              false,
              false,
              false,
              false
            ]
          }
        });

        //println!(
        //    "{}",
        //    serde_json::to_string_pretty(&flat_files.table_metadata).unwrap()
        //);
        assert!(
            json!({"e": [],"main": []}) == serde_json::to_value(&flat_files.table_rows).unwrap()
        );
        assert!(expected_metadata == serde_json::to_value(&flat_files.table_metadata).unwrap());

        flat_files.process_value(myjson.clone());
        flat_files.create_rows().unwrap();

        let expected_metadata = json!(
        {
          "e": {
            "field_type": [
              "text",
              "text",
              "number",
              "text"
            ],
            "fields": [
              "_link",
              "_link_main",
              "ea",
              "eb"
            ],
            "field_counts": [
              4,
              4,
              4,
              4
            ],
            "rows": 4,
            "ignore": false,
            "ignore_fields": [
              false,
              false,
              false,
              false
            ]
          },
          "main": {
            "field_type": [
              "text",
              "text",
              "text",
              "text",
              "text",
              "date"
            ],
            "fields": [
              "_link",
              "_link_main",
              "a",
              "c",
              "d_da",
              "d_db"
            ],
            "field_counts": [
              2,
              2,
              2,
              2,
              2,
              2
            ],
            "rows": 2,
            "ignore": false,
            "ignore_fields": [
              false,
              false,
              false,
              false,
              false,
              false
            ]
          }
        });

        //println!(
        //    "{}",
        //    serde_json::to_string_pretty(&flat_files.table_metadata).unwrap()
        //);
        assert!(
            json!({"e": [],"main": []}) == serde_json::to_value(&flat_files.table_rows).unwrap()
        );
        assert!(expected_metadata == serde_json::to_value(&flat_files.table_metadata).unwrap());
    }

    #[test]
    fn test_inline_o2o_when_o2o () {
        let json1 = json!({
            "id": "1",
            "e": [{"ea": 1, "eb": "eb2"}]
        });
        let json2 = json!({
            "id": "2",
            "e": [{"ea": 2, "eb": "eb2"}],
        });

        let tmp_dir = TempDir::new().unwrap();

        let mut flat_files = FlatFiles::new(
            tmp_dir.path().join("output").to_string_lossy().into_owned(),
            true,
            true,
            true,
            "main".to_string(),
            vec![],
            true,
        )
        .unwrap();

        flat_files.process_value(json1);
        flat_files.process_value(json2);

        flat_files.create_rows().unwrap();
        flat_files.mark_ignore();

        println!(
            "{}",
            serde_json::to_string_pretty(&flat_files.table_metadata).unwrap()
        );
        assert!(flat_files.table_metadata.get("e").unwrap().ignore == true);
        assert!(flat_files.table_metadata.get("main").unwrap().ignore_fields == vec![false,false,false,false,false]);

    }

    #[test]
    fn test_inline_o2o_when_o2m () {
        let json1 = json!({
            "id": "1",
            "e": [{"ea": 1, "eb": "eb2"}]
        });
        let json2 = json!({
            "id": "2",
            "e": [{"ea": 2, "eb": "eb2"}, {"ea": 3, "eb": "eb3"}],
        });

        let tmp_dir = TempDir::new().unwrap();

        let mut flat_files = FlatFiles::new(
            tmp_dir.path().join("output").to_string_lossy().into_owned(),
            true,
            true,
            true,
            "main".to_string(),
            vec![],
            true,
        )
        .unwrap();

        flat_files.process_value(json1);
        flat_files.process_value(json2);

        flat_files.create_rows().unwrap();
        flat_files.mark_ignore();

        println!(
            "{}",
            serde_json::to_string_pretty(&flat_files.table_metadata).unwrap()
        );
        assert!(flat_files.table_metadata.get("e").unwrap().ignore == false);
        assert!(flat_files.table_metadata.get("main").unwrap().ignore_fields == vec![false,false,true,true,false]);
    }
}
