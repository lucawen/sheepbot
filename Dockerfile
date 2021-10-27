FROM rust:1.55 as builder

RUN USER=root cargo new --bin sheepbot
WORKDIR ./sheepbot
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/sheepbot*
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    clang \
    curl \
    git \
    libavcodec-dev \
    libavdevice-dev \
    libavfilter-dev \
    libavformat-dev \
    libavresample-dev \
    libavutil-dev \
    libpostproc-dev \
    libswresample-dev \
    libswscale-dev \
    autoconf \
    autogen \
    libtool \
    automake \
    python3 \
    python3-pip \
    aria2 \
    openssl \
    libssl-dev \
    pkg-config \
    ffmpeg

ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder sheepbot/target/release/sheepbot ${APP}/sheepbot
COPY --from=builder sheepbot/config/ ${APP}/config

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./sheepbot"]