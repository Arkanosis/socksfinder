# socksfinder [![Toolforge](https://img.shields.io/badge/toolforge-soon-990000.svg)](https://socksfinder.toolforge.org/) [![](https://img.shields.io/crates/v/socksfinder.svg)](https://crates.io/crates/socksfinder) [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE) [![Build status](https://travis-ci.org/Arkanosis/socksfinder.svg?branch=master)](https://travis-ci.org/Arkanosis/socksfinder)

**socksfinder** is a search engine for sock puppets on Wikimedia projects.

## Usage

```
Usage: socksfinder build <index>
       socksfinder query [--cooccurrences | --threshold=<threshold>] [--order=<order>] <index> <user>...
       socksfinder -h | --help
       socksfinder --version

Commands:
    build                    Build an index from a MediaWiki XML dump (read on the standard input).
    query                    Search pages modified by several users in the index.

Arguments:
    index                    Index built from a MediaWiki dump.
    user                     User which has modified pages to look for.

Options:
    --cooccurrences          Show the co-occurrences matrix instead of the page names.
    -h, --help               Show this screen.
    --order=<order>          Order of results, none can be faster and consume less memory [default: none].
                             Valid orders: none, count_decreasing, count_increasing, alphabetical.
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

```console
$ curl -s "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
     gunzip |
     socksfinder build frwiki-latest.idx
```

This only needs to be done once, though, and the resulting index can be
redistributed to other users who don't have a fast enough internet access or
a powerful enough computer. For the French Wikipedia, the index is almost
600 MiB big and can be compressed quite efficiently for distribution (less
than 300 MiB when compressed using `gzip --best`).

### Searching for pages modified by editors from a list

Searching for pages modified by one or several editors usually requires only
a very limited amount of memory (by today standards, at least), around 20 or
30 MiB of RAM. It's usually quite fast as well, around 10 to 50 milliseconds
per user depending on your CPU and the number of unique modified pages, though
it can take as much as a few seconds when searching for pages modified by
editors who have modified several hundred thousands of distinct pages.

```console
$ socksfinder query frwiki-latest.idx Arkanosis Arktest Arkbot
Projet:Articles sans portail/1: 3 (Arkanosis, Arktest, Arkbot)
Utilisateur:Arktest/test: 3 (Arkanosis, Arktest, Arkbot)
```

By default, only pages modified by all the users in the list are returned. If
you want pages modified by at least some threshold, use the `--threshold`
option.

```console
$ socksfinder query --threshold=2 frwiki-latest.idx Arkanosis Arktest Arkbot
Utilisateur:Arkbot/Ébauches dans le top 1000: 2 (Arkanosis, Arkbot)
Modèle:Infobox Equipe MotoGP/Bac à sable: 2 (Arkanosis, Arktest)
Projet:Articles sans portail/1: 3 (Arkanosis, Arktest, Arkbot)
Aholfing: 2 (Arktest, Arkbot)
[141 more lines]
```

Instead of the list of modified pages, you can get the co-occurrences matrix,
that is, the matrix of the number of pages modified by each pair of editors
from the list.

```console
$ socksfinder query --cooccurrences frwiki-latest.idx Arkanosis Arktest Arkbot
+-----------+-----------+---------+--------+
|           | Arkanosis | Arktest | Arkbot |
+-----------+-----------+---------+--------+
| Arkanosis |           | 106     | 40     |
+-----------+-----------+---------+--------+
| Arktest   | 106       |         | 3      |
+-----------+-----------+---------+--------+
| Arkbot    | 40        | 3       |        |
+-----------+-----------+---------+--------+
```

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/socksfinder/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/socksfinder/issues).

## License

socksfinder is copyright (C) 2020 Jérémie Roquet <jroquet@arkanosis.net> and
licensed under the ISC license.
