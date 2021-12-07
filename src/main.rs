use anyhow::Result;
use clap::App;
use libflatterer::{flatten, flatten_from_jl, FlatFiles, Selector};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn main() -> Result<()> {
    let matches = App::new("flatterer")
        .version("0.6")
        .author("David Raznick")
        .about("Make JSON flatterer")
        .args_from_usage(
                           "<INPUT>                              'Sets the input file to use'
                           <OUT_DIR>                             'Sets the output directory'
                           -p --path=[path]                      'Key where array lives, leave if array is at root'
                           -j --jl                               'Treat input as JSON Lines, path will be ignored'
                           -c --csv                              'Output csv files (defualt but required if xlsx is selected)'
                           -n --nocsv                            'Do not output csv'
                           -x --xlsx                             'Output xlsx file'
                           -i --inline-one-to-one                'If array always has only one element treat relationship as one-to-one'
                           -f --fields=[file]                    'fields.csv file to determine order and name of fields.'
                           -o --only-fields                      'Use only fields in csv file and no others'
                           -m --main=[main]                      'Table name of top level object'
                           -s --schema=[schema]                  'Link to remote or local JSONSchema to help out with field ordering and naming'
                           -t --table-prefix=[table-prefix]      'Prefix to add to all table names'
                           -a --path-separator=[path-separator]  'Seperator to denote new path within the input JSON. Defaults to `_`'
                           -i --schema-titles=[schema-titles]    'Use titles from JSONSchema in the given way. Options are `full`, `slug`, `underscore_slug`. Default to not using titles..'
                           --force                               'Delete output directory if it exist'",
        )
        .get_matches();

    let input = matches.value_of("INPUT").unwrap(); //ok as parser will detect
    let input_path = PathBuf::from(input);
    if !input_path.exists() {
        eprintln!("Can not find file {}", input);
        return Ok(());
    }
    let input = BufReader::new(File::open(input)?);

    let output_dir = matches.value_of("OUT_DIR").unwrap(); //ok as parser will detect

    let mut selectors = vec![];

    if let Some(path) = matches.value_of("path") {
        selectors.push(Selector::Identifier(format!("\"{}\"", path.to_string())));
    }

    let main_table_name: String;

    if let Some(main) = matches.value_of("main") {
        main_table_name = main.to_string();
    } else {
        main_table_name = "main".to_string();
    }

    let schema_path = if let Some(schema_path) = matches.value_of("schema") {
        schema_path
    } else {
        ""
    };

    let table_prefix = if let Some(table_prefix) = matches.value_of("table-prefix") {
        table_prefix
    } else {
        ""
    };

    let path_separator = if let Some(path_seperator) = matches.value_of("path-separator") {
        path_seperator
    } else {
        "_"
    };

    let schema_titles = if let Some(schema_titles) = matches.value_of("schema-titles") {
        schema_titles
    } else {
        ""
    };

    let mut flat_files = FlatFiles::new(
        output_dir.to_string(),
        !matches.is_present("nocsv"),
        matches.is_present("xlsx"),
        matches.is_present("force"),
        main_table_name,
        vec![],
        matches.is_present("inline-one-to-one"),
        schema_path.to_string(),
        table_prefix.to_string(),
        path_separator.to_string(),
        schema_titles.to_string(),
    )?;

    if let Some(fields) = matches.value_of("fields") {
        flat_files.use_fields_csv(fields.to_string(), matches.is_present("only-fields"))?;
    };

    if matches.is_present("jl") {
        flatten_from_jl(input, flat_files)?;
    } else {
        flatten(input, flat_files, selectors)?;
    }

    Ok(())
}
