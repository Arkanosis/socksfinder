# socksfinder [![](https://img.shields.io/crates/v/socksfinder.svg)](https://crates.io/crates/socksfinder) [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE) [![Build status](https://travis-ci.org/Arkanosis/socksfinder.svg?branch=master)](https://travis-ci.org/Arkanosis/socksfinder)

**socksfinder** is a search engine for sock puppets on Wikimedia projects.

## Usage

```
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
```

## Compiling

Run `cargo build --release` in your working copy.

## Examples

### Building an index from the last dump of the French Wikipedia

```sh
curl -s "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
    gunzip |
    socksfinder build frwiki-latest.idx
```

### Searching for pages modified by at least two editors from a list

```sh
socksfinder query frwiki-latest.idx Arkanosis Arktest Arkbot
```

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/socksfinder/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/socksfinder/issues).

## License

socksfinder is copyright (C) 2020 Jérémie Roquet <jroquet@arkanosis.net> and
licensed under the ISC license.
