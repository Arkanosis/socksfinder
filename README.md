<img src="static/logo.svg" alt="socksfinder logo" align="right" height="180">

# socksfinder [![Toolforge](https://img.shields.io/endpoint?url=https%3A%2F%2Fsocksfinder.toolforge.org%2Fbadge&label=toolforge&color=990000)](https://socksfinder.toolforge.org/) [![](https://img.shields.io/crates/v/socksfinder.svg)](https://crates.io/crates/socksfinder) [![License](https://img.shields.io/badge/license-ISC-blue.svg)](/LICENSE) [![Build status](https://travis-ci.org/Arkanosis/socksfinder.svg?branch=master)](https://travis-ci.org/Arkanosis/socksfinder)

**socksfinder** is a search engine for sock puppets on Wikimedia projects.

## Usage

```
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
    --threshold=<threshold>  Number of different editors, 0 for all of them [default: 0].
    --version                Show version.
```

## Compiling

Run `cargo build --release` in your working copy.

## Examples

### Building an index from the last dump of the French Wikipedia

Building an index can take quite a while and eat a significant amount of memory
(depending on the size of the dump). For the French Wikipedia, it takes about
45 minutes with a fast internet access, and consumes close to 1.5 GiB of RAM.

```console
$ curl -s "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
     gunzip |
     socksfinder build frwiki-latest.idx
```

This only needs to be done once, though, and the resulting index can be
redistributed to other users who don't have a fast enough internet access or
a powerful enough computer. For the French Wikipedia, the index is around
700 MiB big and can be compressed quite efficiently for distribution (less
than 400 MiB when compressed using `gzip --best`).

### Searching for pages modified by editors from a list

Searching for pages modified by one or several editors usually requires only
a very limited amount of memory (by today standards, at least), around 20 or
30 MiB of RAM. It's usually quite fast as well, around 10 to 50 milliseconds
per user depending on your CPU and the number of unique modified pages, though
it can take as much as a few seconds when searching for pages modified by
editors who have modified several hundred thousands of distinct pages (for 
even faster performance, see the [server mode](#server-mode) below).

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

### Server mode

#### Basics

socksfinder can be run in server mode, which means it runs a small HTTP server
to provide an HTML / plain text interface to the same features as when using
the command line.

```console
$ socksfinder serve --hostname=localhost --port=8697 frwiki-latest.idx
```

It can then be used either by opening the HTML interface in a web browser (eg.
http://localhost:8697 in the example above), or by querying it using an HTTP
client.

```console
$ curl 'http://localhost:8697/query?users=Arkanosis,Arktest,Arkbot&coocurrences=true'
```

#### Advantages and downsides

Server mode has the following advantages over command line usage:
 - users don't need to download the program;
 - users don't need to build or download an index;
 - users that don't have the time, tools or technical skills can use it;
 - it's even faster: finding millions of edits can take less than one second
depending on your CPU.

It has however the following downsides:
 - it requires much more memory, about the same as the size of the index (for
the French Wikipedia, it requires around 800 MiB of RAM).

#### Index update with zero downtime

When running in server mode, it's possible to reload the index without any
downtime. This is especially useful when you have built a new version of the
index (for example after a new dump has been made available) and you want to
switch to it without stopping the service.

To reload the index, send an HTTP GET request to `/reload`.

```console
$ curl 'http://localhost:8697/reload'
Index reloaded
```

Reload only happens if the index has changed.

```console
$ curl 'http://localhost:8697/reload'
Index already up-to-date, no need to reload
```

Note that to ensure there's no downtime, socksfinder needs to keep the old
index in memory until the new index has been completely loaded and all
running queries have been answered. Therefore, while reloading the index,
socksfinder can use as much memory as about twice the size of the index.

The recommended way to perform index updates is to have a symlink to the
latest index and to start socksfinder in server mode with that symlink. When
there is a new version of the index available, update the symlink and request
an index reload. Older indexes can then be deleted when disk space is running
out.

```console
$ curl 'http://localhost:8697/version'
Running socksfinder v0.7.0 (frwiki-20220220)
$ ls -al
[…]
-rw-r--r--  1 arkanosis arkanosis 763M 22 févr. 02:42 frwiki-20220220.idx
-rw-r--r--  1 arkanosis arkanosis 771M  2 mars  01:41 frwiki-20220301.idx
lrwxrwxrwx  1 arkanosis arkanosis   19 22 févr. 02:45 frwiki-latest.idx -> frwiki-20220220.idx
$ ln -sf frwiki-20220301.idx frwiki-latest.idx
$ ls -al
[…]
-rw-r--r--  1 arkanosis arkanosis 763M 22 févr. 02:42 frwiki-20220220.idx
-rw-r--r--  1 arkanosis arkanosis 771M  2 mars  01:41 frwiki-20220301.idx
lrwxrwxrwx  1 arkanosis arkanosis   19  2 mars  01:47 frwiki-latest.idx -> frwiki-20220301.idx
$ curl 'http://localhost:8697/reload'
Index reloaded
$ curl 'http://localhost:8697/version'
Running socksfinder v0.7.0 (frwiki-20220301)
```

#### Instance on Toolforge

An instance of socksfinder is available [on Toolforge](https://socksfinder.toolforge.org/).
Its status is visible at [K8s-status](https://k8s-status.toolforge.org/namespaces/tool-socksfinder/).

## Running on Toolforge

### Uploading socksfinder to Toolforge

Copy the socksfinder binary directly to your tool's home directory:

```console
$ rsync -pt ./target/release/socksfinder $LOGIN@login.toolforge.org:/data/project/$PROJECT/
```

Don't forget to replace `$LOGIN` with your Toolforge login and `$PROJECT` with
your tool name.

### Building new indexes as new dumps are available

Create a `data` directory to store the index:

```console
$ mkdir -p data
```

Create the following `$HOME/jobs.yaml` file:

```yaml
---
- name: update-index
  mem: 2Gi
  command: >-
    dump=$(date '+%Y%m%d' -d '2 days ago') ;
    gunzip -c /public/dumps/public/frwiki/${dump}/frwiki-${dump}-stub-meta-history.xml.gz |
      ./socksfinder build ./data/frwiki-${dump}.idx &&
      ln -sf frwiki-${dump}.idx ./data/frwiki-latest.idx &&
      curl https://$PROJECT.toolforge.org/reload
  image: tf-golang111
  schedule: "0 0 3,22 * *"
  emails: all
```

Don't forget to replace `$PROJECT` with your Toolforge project name. You may
also need to adjust the memory limit to what's needed to fit the entire index
in memory.

Then, run the following command:

```console
$ toolforge-jobs load jobs.yaml
```

Index creation jobs will then automatically be started at midnight on the 3rd
and the 22nd of each month, to build and index from the dumps from the 1st and
the 20th of that month.

### Serving the latest index to end-users

Create the following `$HOME/service.template` file:

```yaml
backend: kubernetes
mem: 2Gi
type: golang111
extra_args:
 - /data/project/$PROJECT/socksfinder
 - serve
 - --hostname=0.0.0.0
 - --port=8000
 - /data/project/$PROJECT/data/frwiki-latest.idx
```

Don't forget to replace `$PROJECT` with your Toolforge project name and
`frwiki-latest.idx` with the name of the symlink to your latest index. You may
also need to adjust the memory limit to what's needed to fit the entire index
in memory (twice, if you want index updates with zero downtime).

Then, run the following command:

```console
$ webservice start
```

### Monitoring

Go to your tool's namespace status panel at:

```
https://k8s-status.toolforge.org/namespaces/tool-$PROJECT/
```

Don't forget to replace `$PROJECT` with your tool name.

## Contributing and reporting bugs

Contributions are welcome through [GitHub pull requests](https://github.com/Arkanosis/socksfinder/pulls).

Please report bugs and feature requests on [GitHub issues](https://github.com/Arkanosis/socksfinder/issues).

## License

socksfinder is copyright (C) 2020-2023 Jérémie Roquet <jroquet@arkanosis.net>
and licensed under the ISC license.
