FROM rust:1.42.0

RUN apt-get -y update && apt-get -y install \
        vim
RUN apt-get install -y fzf

WORKDIR /ls-key
