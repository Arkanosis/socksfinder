#! /bin/sh

BASEDIR="$(dirname "$(basename "$(readlink -f "$0")")")"

curl -s "https://dumps.wikimedia.org/frwiki/latest/frwiki-latest-stub-meta-history.xml.gz" |
    gunzip |
    "$BASEDIR/target/debug/socksfinder" build "frwiki-latest.idx" &&
    "$BASEDIR/target/debug/socksfinder" query "frwiki-latest.idx" "Arkanosis" "Arktest" "Arkbot"
