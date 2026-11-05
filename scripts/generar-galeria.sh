#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo build --release --locked
bin=./target/release/lab5-shaders
"$bin" --solo-render --escena sistema --tiempo 0 --salida renders/00-parrilla.png
"$bin" --solo-render --escena estrella --tiempo 0 --salida renders/01-cinco-luces.png
"$bin" --solo-render --escena rocoso --tiempo 0 --salida renders/02-monaco.png
"$bin" --solo-render --escena gaseoso --tiempo 0 --salida renders/03-rebufo.png
"$bin" --solo-render --escena gaseoso --tiempo 0 --ancho 1920 --alto 1200 --salida renders/concurso-rebufo.png
for escena in estrella rocoso gaseoso; do
    "$bin" --escena "$escena" --tiempo 0 --comparar-capas --salida "renders/capas-$escena.png"
done
