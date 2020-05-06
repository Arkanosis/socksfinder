# socksfinder [![](https://img.shields.io/crates/v/socksfinder.svg)](https://crates.io/crates/socksfinder) [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE) [![Build status](https://travis-ci.org/Arkanosis/socksfinder.svg?branch=master)](https://travis-ci.org/Arkanosis/socksfinder)

**socksfinder** is a search engine for sock puppets on Wikimedia projects.

## Usage

```
Usage: socksfinder build <index>
       socksfinder query [--threshold=<threshold>] <index> <user>...
       socksfinder -h | --help
       socksfinder --version

Commands:
    build                    Build an index from a MediaWiki XML dump (read on the standard input).
    query                    Search pages modified by several users in the index.

Arguments:
    index                    Index built from a MediaWiki dump.
    user                     User which has modified pages to look for.

Options:
    -h, --help               Show this screen.
    --threshold=<threshold>  Number of different contributors, 0 for all of them [default: 0].
    --version                Show version.
```

## Compiling

Run `cargo build --release` in your working copy.

## Examples

### Building an index from the last dump of the French Wikipedia

Building an index can take quite a while and eat a significant amount of memory
(depending on the size of the dump). For the French Wikipedia, it takes about
45 minutes with a fast internet access, and consumes close to 500 MiB of RAM.

```sh
curl -s "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
    gunzip |
    socksfinder build frwiki-latest.idx
```

This only needs to be done once, though, and the resulting index can be
redistributed to other users who don't have a fast enough internet access or
a powerful enough computer. For the French Wikipedia, the index is almost
600 Mio big and can be compressed quite efficiently for distribution (less
than 300 Mio when compressed using `gzip --best`.

### Searching for pages modified by editors from a list

Searching for pages modified by one or several editors usually requires only
a very limited amount of memory (by today standards, at least), around 20 or
30 MiB of RAM. It's usually quite fast as well, around 10 to 50 milliseconds
per user depending on your CPU and the number of unique modified pages, though
it can take as much as a few seconds when searching for pages modified by
editors who have modified several hundred thousands of distinct pages.

```sh
socksfinder query frwiki-latest.idx Arkanosis Arktest Arkbot
```

By default, only pages modified by all the users in the list are returned. If
you want pages modified by at least some threashold, use the `--threshold`
option.

```sh
socksfinder query --threshold=2 frwiki-latest.idx Arkanosis Arktest Arkbot
```

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/socksfinder/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/socksfinder/issues).

## License

socksfinder is copyright (C) 2020 Jérémie Roquet <jroquet@arkanosis.net> and
licensed under the ISC license.
