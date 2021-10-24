use clap::App;
use std::path::PathBuf;
use std::io::BufReader;
use std::fs:: File;
use yajlish::ndjson_handler::Selector;
use flatterer::{FlatFiles, flatten_from_jl, flatten};



fn main() -> Result<(), ()> {
    let matches = App::new("flatterer")
        .version("0.1")
        .author("David Raznick")
        .about("Make JSON flatterer")
        .args_from_usage(
                           "<INPUT>               'Sets the input file to use'
                           <OUT_DIR>              'Sets the output directory'
                           -p --path=[path]       'key where array lives, leave if array is at root'
                           -j --jl                'Treat input as JSON Lines, path will be ignored'
                           -c --csv               'Output csv files (defualt but required if xlsx is selected)'
                           -x --xlsx              'Output xlsx file'
                           -i --inline-one-to-one 'If array always has only one element treat relationship as one-to-one'
                           -f --fields=[file]     'fields.csv file to determine order of fields.'
                           -o --only-fields       'use only fields in csv file and no others'
                           -m --main=[main]       'Table name of top level object'
                           --force                'Delete output directory if it exist'",
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let input_path = PathBuf::from(input);
    if !input_path.exists() {
        eprintln!("Can not find file {}", input);
        return Ok(());
    }
    let input = BufReader::new(File::open(input).unwrap());

    let output_dir = matches.value_of("OUT_DIR").unwrap();

    let mut selectors = vec![];

    if let Some(path) = matches.value_of("path") {
        selectors.push(Selector::Identifier(format!("\"{}\"", path.to_string())));
    }

    let main_table_name: String;

    if let Some(main) = matches.value_of("main") {
        main_table_name = main.to_string();
    } else {
        main_table_name = format!("main");
    }

    let mut flat_files = FlatFiles::new (
        output_dir.to_string(),
        matches.is_present("csv") || !matches.is_present("xlsx"),
        matches.is_present("xlsx"),
        matches.is_present("force"),
        main_table_name,
        vec![],
        matches.is_present("inline-one-to-one"),
    ).unwrap();

    if let Some(fields) = matches.value_of("fields") {
        flat_files.use_fields_csv(fields.to_string(), matches.is_present("only-fields")).unwrap();
    };

    if matches.is_present("jl") {
        flatten_from_jl(input, flat_files).unwrap();
    } else {
        flatten(input, flat_files, selectors).unwrap();
    }

    Ok(())
}
