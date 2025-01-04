#!/bin/sh -eu

set -o pipefail

name="$1"

cd "$(dirname "$0")/transpiler"

out="ast-${1//_/-}.c"

echo "extern ast_node_t *token_stream_$name(token_stream_t *self);" 

if [ -f "$out" ]; then
  echo "File already exists: $out"
  exit 1
fi

echo "create $out"
cat ast-node-implementation.c.template | sed "s/path/$name/g" > "$out"

