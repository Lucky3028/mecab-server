# syntax=docker/dockerfile:1.4
FROM ubuntu:22.04 as dic
RUN apt-get update \
    && apt-get install -y g++ git make curl file xz-utils sudo mecab libmecab-dev mecab-ipadic-utf8 \
    && cd /var \
    && git clone https://github.com/neologd/mecab-ipadic-neologd.git --depth=1 \
    && cd mecab-ipadic-neologd \
    && ./bin/install-mecab-ipadic-neologd -y -n -a \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# clux/muslrust doesn't release 1.64.0 stable image at 2022/10/02
FROM clux/muslrust:1.64.0-nightly-2022-08-06 AS chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY --link . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
RUN apt-get update \
    && apt-get install -y mecab libmecab-dev mecab-ipadic-utf8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner --link /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY --link . .
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM debian:stable-slim

RUN apt-get update \
    && apt-get install -y mecab libmecab-dev mecab-ipadic-utf8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=dic --link /usr/lib/x86_64-linux-gnu/mecab/dic/mecab-ipadic-neologd /usr/lib/x86_64-linux-gnu/mecab/dic/
COPY --from=build --link /app/target/x86_64-unknown-linux-musl/release/mecab-server /mecab-server
ENV NEOLOGD_DIC_PATH /usr/lib/x86_64-linux-gnu/mecab/dic/mecab-ipadic-neologd

ENTRYPOINT ["/mecab-server"]
