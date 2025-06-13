FROM buildpack-deps:jammy

LABEL org.opencontainers.image.source=https://github.com/rust-lang/docker-rust

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.87.0

RUN set -eux; \
    dpkgArch="$(dpkg --print-architecture)"; \
    case "${dpkgArch##*-}" in \
        amd64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='20a06e644b0d9bd2fbdbfd52d42540bdde820ea7df86e92e533c073da0cdd43c' ;; \
        armhf) rustArch='armv7-unknown-linux-gnueabihf'; rustupSha256='3b8daab6cc3135f2cd4b12919559e6adaee73a2fbefb830fadf0405c20231d61' ;; \
        arm64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='e3853c5a252fca15252d07cb23a1bdd9377a8c6f3efa01531109281ae47f841c' ;; \
        i386) rustArch='i686-unknown-linux-gnu'; rustupSha256='a5db2c4b29d23e9b318b955dd0337d6b52e93933608469085c924e0d05b1df1f' ;; \
        *) echo >&2 "unsupported architecture: ${dpkgArch}"; exit 1 ;; \
    esac; \
    url="https://static.rust-lang.org/rustup/archive/1.28.2/${rustArch}/rustup-init"; \
    wget "$url"; \
    echo "${rustupSha256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain $RUST_VERSION --default-host ${rustArch}; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;



RUN apt update -y && \
	apt install -y \
	libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev

RUN curl -fsSL "https://deb.nodesource.com/setup_20.x" | bash -
RUN apt-get install -y nodejs



#RUN apt install -y curl

#RUN curl https://create.tauri.app/sh | sh








#RUN apt-get update
#RUN apt-get install -y \
#	curl \
#	psmisc

## Install Node.js
#RUN curl -fsSL "https://deb.nodesource.com/setup_20.x" | bash -
#RUN apt-get install -y nodejs

## Install Yarn
#RUN corepack enable

## Install Tarpaulin
#RUN cargo install cargo-tarpaulin

## Install Tauri v1 dependencies
## https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-linux
#RUN apt-get install -y \
#	libwebkit2gtk-4.0-dev \
#	build-essential \
#	curl \
#	wget \
#	libssl-dev \
#	libgtk-3-dev \
#	libayatana-appindicator3-dev \
#	librsvg2-dev

## Tauri v2 dependencies
## => Impossible: https://github.com/tauri-apps/tauri/issues/9039

## Install tauri-driver dependencies
#RUN apt-get install -y \
#	dbus-x11 \
#	webkit2gtk-4.0-dev \
#	webkit2gtk-driver \
#	xvfb

## Install tauri-driver
## https://tauri.app/v1/guides/testing/webdriver/introduction#system-dependencies
#RUN cargo install tauri-driver

#COPY ../ /app

WORKDIR /app

CMD ["npm", "run", "tauri", "build"]

#RUN cd /app \
	#npm run tauri build

