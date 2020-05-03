#! /bin/sh

BASEDIR="$(dirname "$(basename "$(readlink -f "$0")")")"

curl "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
    gunzip |
    "$BASEDIR/target/release/socksfinder" build "frwiki-latest.idx" > /dev/null
