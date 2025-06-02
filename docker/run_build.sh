#! /bin/bash

docker build -t stdbuild:latest -o type=local,dest=/tmp/stdbuild.img .

docker run -it --rm /tmp/stdbuild.img
