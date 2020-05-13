use std::{
    io::{
        BufReader,
        BufWriter,
        Write,
    },
    process,
};

use serde_derive::Deserialize;

use std::fs::File;

const USAGE: &str = "
Usage: socksfinder build <index>
       socksfinder query [--cooccurrences | --threshold=<threshold>] [--order=<order>] <index> <user>...
       socksfinder serve [--hostname=<hostname>] [--port=<port>] <index>
       socksfinder stats <index>
       socksfinder -h | --help
       socksfinder --version

Commands:
    build                    Build an index from a MediaWiki XML dump (read on the standard input).
    query                    Search pages modified by several users in the index.
    serve                    Start a small HTTP server to serve the index.
    stats                    Display statistics about the index.

Arguments:
    index                    Index built from a MediaWiki dump.
    user                     User which has modified pages to look for.

Options:
    --cooccurrences          Show the co-occurrences matrix instead of the page names.
    -h, --help               Show this screen.
    --hostname=<hostname>    Hostname to resolve to find the network interface to serve the index [default: localhost].
    --order=<order>          Order of results, none can be faster and consume less memory [default: none].
                             Valid orders: none, count_decreasing, count_increasing, alphabetical.
    --port=<port>            Port on which to serve the index [default: 8080].
    --threshold=<threshold>  Number of different contributors, 0 for all of them [default: 0].
    --version                Show version.
";

#[derive(Deserialize)]
struct Args {
    cmd_build: bool,
    cmd_query: bool,
    cmd_serve: bool,
    cmd_stats: bool,
    arg_index: String,
    arg_user: Vec<String>,
    flag_cooccurrences: bool,
    flag_hostname: String,
    flag_order: socksfinder::Order,
    flag_port: u16,
    flag_threshold: usize,
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
        if args.cmd_build {
            let output = File::create(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            let mut buffered_output = BufWriter::new(output);
            if socksfinder::build(&mut std::io::stdin().lock(), &mut buffered_output).is_err() ||
               buffered_output.flush().is_err() {
                process::exit(1);
            }
        } else if args.cmd_query {
            let input = File::open(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            let mut buffered_input = BufReader::new(input);
            let mut output = std::io::stdout();
            if socksfinder::query(&mut buffered_input, &mut output, &args.arg_user, if args.flag_threshold != 0 { args.flag_threshold } else { args.arg_user.len() }, args.flag_order, args.flag_cooccurrences).is_err() ||
               output.flush().is_err() {
                process::exit(1);
            }
        } else if args.cmd_serve {
            let input = File::open(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            if socksfinder::serve(input, args.flag_hostname, args.flag_port).is_err() {
                process::exit(1);
            }
        } else if args.cmd_stats {
            let input = File::open(&args.arg_index).unwrap_or_else(|cause| {
                eprintln!("socksfinder: can't open index: {}: {}", &args.arg_index, &cause);
                process::exit(1);
            });
            let mut buffered_input = BufReader::new(input);
            if socksfinder::stats(&mut buffered_input).is_err() {
                process::exit(1);
            }
        }
    }
}
