use std::{
    io::{
        BufReader,
        BufWriter,
    },
    process,
};

use serde_derive::Deserialize;

use std::fs::File;

const USAGE: &str = "
Usage: socksfinder build <index>
       socksfinder query <index> <user>...
       socksfinder -h | --help
       socksfinder --version

Commands:
    build        Build an index from a MediaWiki XML dump (read on the standard input).
    query        Search pages modified by several users in the index.

Arguments:
    index        Index built from a MediaWiki dump.
    user         User which has modified pages to look for.

Options:
    -h, --help   Show this screen.
    --version    Show version.
";

#[derive(Deserialize)]
struct Args {
    cmd_build: bool,
    cmd_query: bool,
    arg_index: String,
    arg_user: Vec<String>,
    flag_version: bool,
}

fn main() {
    let args: Args =
        docopt::Docopt::new(USAGE)
            .and_then(|docopts|
                docopts.argv(std::env::args().into_iter())
                   .deserialize()
            )
            .unwrap_or_else(|error|
                error.exit()
            );

    if args.flag_version {
        println!("socksfinder v{}", socksfinder::version());
    } else {
        let result = if args.cmd_build {
            let output = File::create(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            let mut buffered_output = BufWriter::new(output);
            socksfinder::build(&mut std::io::stdin().lock(), &mut buffered_output)
        } else if args.cmd_query {
            let input = File::open(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            let mut buffered_input = BufReader::new(input);
            socksfinder::query(&mut buffered_input, &args.arg_user)
        } else {
            Ok(())
        };
        if result.is_err() {
            process::exit(1);
        }
    }
}
