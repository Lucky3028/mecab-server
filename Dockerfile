# syntax=docker/dockerfile:1.4
FROM rust:1.64.0-slim AS chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY --link . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
RUN apt-get update \
    && apt-get install -y g++ git make curl sudo file xz-utils mecab libmecab-dev mecab-ipadic-utf8 \
    && cd /var \
    && git clone https://github.com/neologd/mecab-ipadic-neologd.git --depth=1 \
    && cd mecab-ipadic-neologd \
    && ./bin/install-mecab-ipadic-neologd -y -n -a

COPY --from=planner --link /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY --link . .
RUN cargo build --target x86_64-unknown-linux-musl --release

### Runner ###
FROM gcr.io/distroless/cc
COPY --from=build --link /usr/local/lib/mecab /usr/local/lib/
COPY --from=build --link /usr/local/bin/mecab /usr/local/bin/
COPY --from=build --link /app/target/x86_64-unknown-linux-musl/release/mecab-server /mecab-server

USER nonroot
ENV NEOLOGD_DIC_PATH /usr/local/lib/mecab/dic/mecab-ipadic-neologd

ENTRYPOINT ["/mecab-server"]
