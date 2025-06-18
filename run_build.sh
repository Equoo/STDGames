#! /bin/bash

docker build -t stdbuild:latest docker/release

docker run -it --rm \
  -v $PWD/:/app \
  -v $PWD/src-tauri/target:/app/src-tauri/target \
  stdbuild
  #-v $HOME/.cargo:/usr/local/cargo \
  #-v $HOME/.rustup:/usr/local/rustup \