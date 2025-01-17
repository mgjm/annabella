#!/bin/sh -eu

cargo build

for file in ~/gcc-3.1.0/gcc/ada/*.ad?; do
  case "$file" in
    *.adt) continue ;;
  esac

  target/debug/annabella-rs "$file"
  # break
done

echo done
