use clap::{arg, ArgAction, Command};
use env_logger::Env;
use eyre::Result;
use libflatterer::{flatten_all, Options, TERMINATE};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

fn main() -> Result<()> {
    let matches = Command::new("flatterer")
        .version("0.14")
        .author("David Raznick")
        .about("Make JSON flatterer")
        // `-h` is used by --schema-titles below, so expose help as `--help` only.
        .disable_help_flag(true)
        .arg(arg!(--help "Print help").action(ArgAction::Help).required(false))
        .args(&[
            arg!(<INPUT>                               "Sets the input file to use"),
            arg!(<OUT_DIR>                             "Sets the output directory"),
            arg!(-p --path [path]                      "Key where array lives, leave if array is at root"),
            arg!(-j --ndjson                           "Treat input as JSON Lines, path will be ignored"),
            arg!(--"json-stream"                       "Treat input as stream of JSON objects"),
            arg!(-c --csv                              "Output csv files (defualt but required if xlsx is selected)"),
            arg!(-n --nocsv                            "Do not output csv"),
            arg!(-x --xlsx                             "Output xlsx file"),
            arg!(-q --sqlite                           "Output sqlite db"),
            arg!(-u --parquet                          "Output parquet files"),
            arg!(-i --"inline-one-to-one"              "If array always has only one element treat relationship as one-to-one"),
            arg!(-f --fields [file]                    "fields.csv file to determine order and name of fields."),
            arg!(-o --"only-fields"                    "Use only fields in csv file and no others"),
            arg!(-b --tables [file]                    "tables.csv file to determine name and order of tables."),
            arg!(-l --"only-tables"                    "Use only tables in csv file and no others"),
            arg!(-m --main [main]                      "Table name of top level object"),
            arg!(-s --schema [schema]                  "Link to remote or local JSONSchema to help out with field ordering and naming"),
            arg!(-t --"table-prefix" [tableprefix]     "Prefix to add to all table names"),
            arg!(-a --"path-separator" [pathseparator] "Seperator to denote new path within the input JSON. Defaults to `_`"),
            arg!(-h --"schema-titles" [schematitles]   "Use titles from JSONSchema in the given way. Options are `full`, `slug`, `underscore_slug`. Default to not using titles.."),
            arg!(--threads [threads]                   "Number of threads to use"),
            arg!(--force                               "Delete output directory if it exist"),
        ])
        .get_matches();

    env_logger::Builder::from_env(Env::new().filter_or("FLATTERER_LOG", "info"))
        .format_timestamp_millis()
        .format_target(false)
        .init();

    ctrlc::set_handler(|| {
        TERMINATE.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let input = matches.get_one::<String>("INPUT").unwrap(); //ok as parser will detect
    let input_path = PathBuf::from(input);
    if !input_path.exists() {
        eprintln!("Can not find file {}", input);
        return Ok(());
    }

    let output_dir = matches.get_one::<String>("OUT_DIR").unwrap(); //ok as parser will detect

    let mut options = Options::builder().build();

    let mut selectors = vec![];

    if let Some(path) = matches.get_one::<String>("path") {
        selectors.push(path.to_string());
    }
    options.path = selectors;

    options.main_table_name = if let Some(main) = matches.get_one::<String>("main") {
        main.to_string()
    } else {
        "main".to_string()
    };

    options.schema = if let Some(schema_path) = matches.get_one::<String>("schema") {
        schema_path.into()
    } else {
        "".into()
    };

    options.table_prefix = if let Some(table_prefix) = matches.get_one::<String>("table-prefix") {
        table_prefix.into()
    } else {
        "".into()
    };

    options.path_separator = if let Some(path_seperator) = matches.get_one::<String>("path-separator") {
        path_seperator.into()
    } else {
        "_".into()
    };

    options.schema_titles = if let Some(schema_titles) = matches.get_one::<String>("schema-titles") {
        schema_titles.into()
    } else {
        "".into()
    };

    options.csv = !matches.get_flag("nocsv");
    options.xlsx = matches.get_flag("xlsx");
    options.sqlite = matches.get_flag("sqlite");
    options.parquet = matches.get_flag("parquet");
    options.force = matches.get_flag("force");
    options.ndjson = matches.get_flag("ndjson");
    options.json_stream = matches.get_flag("json-stream");
    options.inline_one_to_one = matches.get_flag("inline-one-to-one");

    if let Some(threads) = matches.get_one::<String>("threads").and_then(|t| t.parse().ok()) {
        options.threads = threads;
    };

    if let Some(fields) = matches.get_one::<String>("fields") {
        options.fields_csv = fields.into();
    };

    if let Some(tables) = matches.get_one::<String>("tables") {
        options.tables_csv = tables.into();
    };

    flatten_all(vec![input.into()], output_dir.into(), options)?;

    log::info!("All finished with no errors!");

    Ok(())
}
