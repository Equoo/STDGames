#! /bin/bash

docker build -t stddev:latest docker/debug

xhost +local:docker

docker run --cap-add=SYS_ADMIN --cap-add=SYS_CHROOT --privileged -it --rm \
  --ipc=host \
  -v $PWD/:/app \
  -v $PWD/src-tauri/target:/app/src-tauri/target \
  -v /tmp/.X11-unix:/tmp/.X11-unix \
  -v /dev/dri:/dev/dri \
  -v /run/user/$UID/at-spi/bus_0:/run/user/$UID/at-spi/bus_0 \
  -v /run/user/$UID/pulse:/run/user/0/pulse \
  -v /sgoinfre:/sgoinfre \
  -v /goinfre:/goinfre \
  -v /tmp:/tmp \
  -v $HOME/.cargo:/usr/local/cargo \
  -v $HOME/.rustup:/usr/local/rustup \
  -e USER=$USER \
  -e GDK_BACKEND=x11 \
  -e GDK_SCALE=1 \
  -e GTK_MODULES='' \
  -e NO_AT_BRIDGE=1 \
  -e QT_X11_NO_MITSHM=1 \
  -e GDK_USE_X11=1 \
  -e GDK_DISABLE_MITSHM=1 \
  -e DISPLAY=$DISPLAY \
  stddev
