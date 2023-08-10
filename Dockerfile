FROM ghcr.io/cross-rs/arm-unknown-linux-gnueabihf:latest

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y \
        curl \
        pkg-config \
        libasound2-dev \
        libasound2-dev:armhf \
        libjack-jackd2-dev \
        libjack-jackd2-dev:armhf \
        gcc \
        gcc-arm-linux-gnueabihf \
        g++ \
        ca-certificates \
        qemu-user \
        make \
        file \
        clang \
