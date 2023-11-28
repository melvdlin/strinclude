use clap::parser::ValueSource;
use clap::{value_parser, Arg, ArgAction, Command};
use itertools::Itertools;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let mut cli = Command::new("strinclude")
        .arg(
            Arg::new("file")
                .index(1)
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .help("the file to generate a header from"),
        )
        .arg(
            Arg::new("out")
                .short('o')
                .long("out")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .conflicts_with_all(["outname", "outdir"])
                .default_value("<file>.h")
                .help("the path to the output file"),
        )
        .arg(
            Arg::new("outname")
                .short('n')
                .long("outname")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .default_value("<file>.h")
                .help("the name of the output file"),
        )
        .arg(
            Arg::new("outdir")
                .short('d')
                .long("outdir")
                .value_parser(value_parser!(PathBuf))
                .action(ArgAction::Set)
                .default_value(".")
                .help("the directory of the output file"),
        )
        .arg(
            Arg::new("varname")
                .short('v')
                .long("varname")
                .value_parser(value_parser!(String))
                .action(ArgAction::Set)
                .help("the name of the string variable in the emitted header file; defaults to <file>, stripped of its extension"),
        )
        .arg(
            Arg::new("terminate")
                .short('t')
                .long("terminate")
                .value_parser(value_parser!(bool))
                .action(ArgAction::SetTrue)
                .help(
                    "add a null character to the end of the generated string definition",
                ),
        );
    cli.build();
    let matches = cli.get_matches();
    let out_source = matches.value_source("out").expect("default");

    let file: PathBuf = matches
        .get_one::<PathBuf>("file")
        .expect("required")
        .clone();

    let out: PathBuf = if out_source != ValueSource::DefaultValue {
        matches.get_one::<PathBuf>("out").expect("default").clone()
    } else {
        let out_name_source = matches.value_source("outname").expect("default");
        let out_name: PathBuf = if out_name_source != ValueSource::DefaultValue {
            matches
                .get_one::<PathBuf>("outname")
                .expect("default")
                .to_owned()
        } else {
            let mut name: OsString = file
                .file_name()
                .ok_or(format!("{} is not a valid file path", file.display()))?
                .to_owned();
            name.push(".h");
            PathBuf::from(name)
        };
        let out_dir: &PathBuf = matches.get_one("outdir").expect("default");
        out_dir.join(out_name)
    };

    let var_name = matches
        .get_one::<String>("varname")
        .map(ToOwned::to_owned)
        .unwrap_or(
            file.file_stem()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
                .to_owned(),
        );

    if strinclude::symbol_name_is_legal(&var_name) {
        return Err(format!("{var_name} is not a legal C variable name!").into());
    }

    let terminate = matches.get_flag("terminate");

    let mut file = File::open(file)?;
    let mut out = File::open(out)?;

    let mut file_content = Vec::new();
    file.read_to_end(&mut file_content)?;

    if terminate {
        file_content.push(0);
    }

    let header_content =
        strinclude::literalize(&var_name, file_content.into_iter()).collect_vec();

    out.write_all(&header_content)?;

    Ok(())
}
