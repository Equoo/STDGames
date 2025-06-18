#! /bin/bash

docker build -t stddev:latest docker/debug

docker run -it --rm \
  -v $PWD/:/app \
  -v $PWD/src-tauri/target:/app/src-tauri/target \
  stddev
#  -v $HOME/.cargo:/usr/local/cargo \
#  -v $HOME/.rustup:/usr/local/rustup \

npm run dev
src-tauri/target/debug/stdgames