alias p := play
alias d := dev

play:
  cargo run

dev:
  cargo run --features dev

build-client target="native":
  cargo build --release {{ if target == "windows-gnu" { "--target=x86_64-pc-windows-gnu" } else { "" } }}

build-apworld:
  #!/usr/bin/env bash
  set -e

  dir=$(mktemp -d)
  tmpgamedir="$dir/suikapelagame"

  mkdir "$tmpgamedir"

  for path in apworld/*; do
    file=$(basename $path)
    cp $path "$tmpgamedir/$file"
  done

  out="$(pwd)/suikapelagame.apworld"
  cd $dir && zip -r "$out" .
