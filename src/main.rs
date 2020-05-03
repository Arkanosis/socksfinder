use serde_derive::Deserialize;

const USAGE: &str = "
Usage: socksfinder build
       socksfinder query <user>...
       socksfinder -h | --help
       socksfinder --version

Commands:
    build        Build an index from the latest dump.
    query        Search pages modified by several users in the index.

Arguments:
    user         User which has modified pages to look for.

Options:
    -h, --help   Show this screen.
    --version    Show version.
";

#[derive(Deserialize)]
struct Args {
    cmd_build: bool,
    cmd_query: bool,
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
    } else if args.cmd_build {
        println!("building index");
        unimplemented!();
    } else if args.cmd_query {
        println!("querying users:");
        for user in &args.arg_user {
            println!("\t{}", user);
        }
        unimplemented!();
    }
}
