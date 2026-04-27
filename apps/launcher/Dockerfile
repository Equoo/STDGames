FROM archlinux:base

RUN pacman -Syu --noconfirm && \
    pacman -S --noconfirm \
        sshfs \
        fuse3 \
        rsync \
        git \
        bubblewrap \
        bash && \
    pacman -Scc --noconfirm

USER root
WORKDIR /
