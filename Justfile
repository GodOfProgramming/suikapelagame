alias p := play
alias d := dev

play:
  cargo run

dev:
  cargo run --features dev

build-client target="native":
  echo "building {{target}}"
  cargo build --release {{ if target == "windows-gnu" { "--target=x86_64-pc-windows-gnu" } else { "" } }}

build-apworld:
  #!/usr/bin/env bash
  set -e

  dir=$(mktemp -d)
  builddir="$dir/suikapelagame-apworld"

  mkdir "$builddir"

  for path in apworld/*; do
    file=$(basename $path)
    cp $path "$builddir/$file"
  done

  out="$(pwd)/suikapelagame.apworld"
  cd $dir && zip -r "$out" .

package target="native": (build-client target) && build-apworld
  #!/usr/bin/env bash
  set -e

  dir=$(mktemp -d)
  builddir="$dir/suikapelagame"

  mkdir "$builddir"

  echo "Building in ${builddir}"

  if [[ "{{target}}" == "native" ]]; then
    bin_name=$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].targets[] | select(.kind[] == "bin") | .name' | head -n 1);
    ln -s "$(pwd)/target/release/${bin_name}" "${builddir}/${bin_name}"
  elif [[ "{{target}}" == "windows-gnu" ]]; then
    ln -s "$(pwd)/target/x86_64-pc-windows-gnu/release/suikapelagame.exe" "${builddir}/suikapelagame.exe"
  fi

  ln -s "$(pwd)/assets" "${builddir}/assets"
  ln -s "$(pwd)/suikapelagame.apworld" "${builddir}/suikapelagame.apworld"

  curdir=$(pwd)
  cd "${builddir}"
  zip -r9 "${curdir}/suikapelagame.zip" ./*
