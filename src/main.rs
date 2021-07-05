use clap::App;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs:: File;
use yajlish::ndjson_handler::{NdJsonHandler, Selector};
use flaterer::FlatFiles;
use serde_json::{Deserializer, Value};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use yajlish::Parser;
use std::io::{self, Write};
use std::mem;

struct JLWriter {
    pub buf: Vec<u8>,
    pub buf_sender: Sender<Vec<u8>>,
}

impl Write for JLWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf == [b'\n'] {
            let mut new_buf = Vec::with_capacity(self.buf.capacity());

            mem::swap(&mut self.buf, &mut new_buf);

            self.buf_sender.send(new_buf).unwrap();

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


fn main() -> Result<(), ()> {
    let matches = App::new("flatterer")
        .version("0.1")
        .author("David Raznick")
        .about("Make JSON flatterer")
        .args_from_usage(
            "<INPUT>             'Sets the input file to use'
                           <OUT_DIR>           'Sets the output directory'
                           -p --path=[path]    'key where array lives, leave if array is at root'
                           -j --jl             'Treat input as JSON Lines, path will be ignored'
                           -m --main=[main]    'Table name of top level object'
                           -f --force          'Delete output directory if it exist'",
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let input_path = PathBuf::from(input);
    if !input_path.exists() {
        eprintln!("Can not find file {}", input);
        return Ok(());
    }
    let mut input = BufReader::new(File::open(input).unwrap());

    let output_dir = matches.value_of("OUT_DIR").unwrap();

    let mut selectors = vec![];

    if let Some(path) = matches.value_of("path") {
        selectors.push(Selector::Identifier(path.to_string()))
    }

    let main_table_name: String;

    if let Some(main) = matches.value_of("main") {
        main_table_name = main.to_string();
    } else {
        main_table_name = format!("main");
    }

    let mut flat_files = FlatFiles::new (
        output_dir.to_string(),
        matches.is_present("force"),
        main_table_name,
        vec![],
    ).unwrap();

    if !matches.is_present("jl") {
        let (buf_sender, buf_receiver) = channel();

        thread::spawn(move || {
            let mut jl_writer = JLWriter {
                buf: vec![],
                buf_sender,
            };

            let mut handler = NdJsonHandler::new(&mut jl_writer, selectors);
            let mut parser = Parser::new(&mut handler);

            parser.parse(&mut input).unwrap();
        });

        for buf in buf_receiver.iter() {
            let value = serde_json::from_slice::<Value>(&buf).unwrap();
            flat_files.process_value(value).unwrap();
        }
    } else {
        let (value_sender, value_receiver) = channel();

        thread::spawn(move || {
            let stream = Deserializer::from_reader(input).into_iter::<Value>();
            for value in stream {
                value_sender.send(value.unwrap()).unwrap();
            }
        });

        for value in value_receiver {
            flat_files.process_value(value).unwrap();
        }
    }
    flat_files.write_files().unwrap();

    Ok(())
}
