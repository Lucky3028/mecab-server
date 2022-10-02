# syntax=docker/dockerfile:1.4
### Prepare for Runner ###
FROM ubuntu:22.04 AS prepare

RUN apt-get update \
    && apt-get install -y g++ git make curl sudo file xz-utils mecab libmecab-dev mecab-ipadic-utf8 \
    && cd /var \
    && git clone https://github.com/neologd/mecab-ipadic-neologd.git --depth=1 \
    && cd mecab-ipadic-neologd \
    && ./bin/install-mecab-ipadic-neologd -y -n -a

### Builder ###
# muslrust/1.64.0 has not been released yet at 2022/10/02
FROM clux/muslrust:1.64.0-nightly-2022-08-06 AS chef

RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY --link . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
COPY --from=planner --link /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY --from=prepare --link /usr/local/lib /usr/local/
COPY --link . .
RUN cargo build --target x86_64-unknown-linux-musl --release

### Runner ###
FROM gcr.io/distroless/cc
USER nonroot
COPY --from=build --link /app/target/x86_64-unknown-linux-musl/release/c-presentation /mecab-server

ENTRYPOINT ["/mecab-server"]
