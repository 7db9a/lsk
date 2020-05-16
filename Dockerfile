FROM rust:1.42.0

RUN apt-get -y update && apt-get -y install \
        vim

WORKDIR /ls-key
