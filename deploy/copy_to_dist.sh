#!/usr/bin/env bash

cp -R ./public/* ${1:-"dist"}

cp ./src/lib/rlottie/rlottie-wasm.wasm ${1:-"dist"}

cp ./node_modules/opus-recorder/dist/decoderWorker.min.wasm ${1:-"dist"}

cp -R ./node_modules/emoji-data-ios/img-apple-64 ${1:-"dist"}
cp -R ./node_modules/emoji-data-ios/img-apple-160 ${1:-"dist"}
