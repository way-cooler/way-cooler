#!/bin/sh

man() {
  file=$1
  dir=$2

  lang="$(dirname $file)"
  basename=$(basename $file .md)
  section=${basename##*.}

  mkdir -p "$dir/$lang/man$section"
  echo "$file > $dir/$lang/man$section/$basename.gz"
  asciidoctor -d manpage -b manpage -o - "$file" | gzip > "$dir/$lang/man$section/$basename.gz"
}

html() {
  file=$1
  dir=$2
  basename=$(basename $file .md)

  lang="$(dirname $file)"
  out="$dir/$lang/$basename.html"

  mkdir -p "$dir/$lang"
  echo "$file > $out"
  asciidoctor -o - "$file" > "$out"
}

# defaults, do all
do_man=true
do_html=true

for i in "$@"; do
  case $i in
    -m|--manualonly)
      do_html=false
    ;;
    -h|--htmlonly)
      do_man=false
    ;;
    *)
      if [ -z "$input" ]; then
        input=$i
      else
        output="$PWD/$i"
      fi
    ;;
  esac
done

which "asciidoctor" > /dev/null
if [ "$?" -eq 1 ]; then
  echo "The asciidoctor executable is required to build man files"
  exit 1
fi

if [ -z "$output" -o -z "$input" ]; then
  echo "Way-cooler's documentation maker\nUsage: ./makedocs.sh [-m|--manualonly] [-h|--htmlonly] input output"
  exit 1
fi

cd $input
for file in */*.md *.md; do
  [ "$do_html" = true ] && html $file $output
  [ "$do_man" = true ] && man $file $output
done
cd - > /dev/null