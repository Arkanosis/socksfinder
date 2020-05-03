# socksfinder [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE)

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

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/socksfinder/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/socksfinder/issues).

## License

socksfinder is copyright (C) 2020 Jérémie Roquet <jroquet@arkanosis.net> and
licensed under the ISC license.
