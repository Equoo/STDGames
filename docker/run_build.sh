#! /bin/bash

docker build -t stdbuild:latest .

docker run -it --rm -v $PWD/../:/app -v $PWD/result:/app/src-tauri/target stdbuild

#docker run -it --rm /tmp/stdbuild.img
# -o type=local,dest=/tmp/stdbuild.img
