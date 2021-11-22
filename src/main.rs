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
                           "<INPUT>                              'Sets the input file to use'
                           <OUT_DIR>                             'Sets the output directory'
                           -p --path=[path]                      'Key where array lives, leave if array is at root'
                           -j --jl                               'Treat input as JSON Lines, path will be ignored'
                           -c --csv                              'Output csv files (defualt but required if xlsx is selected)'
                           -x --xlsx                             'Output xlsx file'
                           -i --inline-one-to-one                'If array always has only one element treat relationship as one-to-one'
                           -f --fields=[file]                    'fields.csv file to determine order of fields.'
                           -o --only-fields                      'Use only fields in csv file and no others'
                           -m --main=[main]                      'Table name of top level object'
                           -s --schema=[schema]                  'Link to remote or local JSONSchema to help out with field ordering'
                           -t --table-prefix=[table-prefix]      'Prefix to add to all table names'
                           -a --path-separator=[path-separator]  'Seperator to denote new path within the input JSON. Defaults to `_`'
                           --force                               'Delete output directory if it exist'",
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

    let schema_path = if let Some(schema_path) = matches.value_of("schema") {
        schema_path
    } else {""};

    let table_prefix = if let Some(table_prefix) = matches.value_of("table-prefix") {
        table_prefix
    } else {""};

    let path_separator = if let Some(path_seperator) = matches.value_of("path-separator") {
        path_seperator
    } else {"_"};

    let mut flat_files = FlatFiles::new (
        output_dir.to_string(),
        matches.is_present("csv") || !matches.is_present("xlsx"),
        matches.is_present("xlsx"),
        matches.is_present("force"),
        main_table_name,
        vec![],
        matches.is_present("inline-one-to-one"),
        schema_path.to_string(),
        table_prefix.to_string(),
        path_separator.to_string()
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
