# syntax=docker/dockerfile:1.4
# clux/muslrust doesn't release 1.64.0 stable image at 2022/10/02
FROM clux/muslrust:1.64.0-nightly-2022-08-06 AS chef
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY --link . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS build
RUN apt-get update \
    && apt-get install -y g++ git make curl sudo file xz-utils mecab libmecab-dev mecab-ipadic-utf8 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner --link /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

COPY --link . .
RUN cargo build --target x86_64-unknown-linux-musl --release

### Runner ###
FROM gcr.io/distroless/cc:debug

WORKDIR /var
RUN apk update \
    && apk --update --no-cache add -y g++ git make curl sudo file xz-utils mecab libmecab-dev mecab-ipadic-utf8
RUN git clone https://github.com/neologd/mecab-ipadic-neologd.git --depth=1 \
    && ./mecab-ipadic-neologd/bin/install-mecab-ipadic-neologd -y -n -a
RUN apk del --purge -y g++ git make curl sudo file xz-utils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /
COPY --from=build --link /app/target/x86_64-unknown-linux-musl/release/mecab-server /mecab-server

USER nonroot
ENV NEOLOGD_DIC_PATH /usr/lib/x86_64-linux-gnu/mecab/dic/mecab-ipadic-neologd

ENTRYPOINT ["/mecab-server"]
