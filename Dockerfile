FROM ubuntu:20.04

ENV HOME=/home/rai
COPY ./ $HOME/toolkit
WORKDIR $HOME/toolkit
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && apt-get install -y tzdata \
    && dpkg-reconfigure --frontend noninteractive tzdata \
    && apt-get install -y curl build-essential postgresql postgresql-postgis libssl-dev pkg-config
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 0.43 | sh -s -- -y

ENV PATH="$HOME/.cargo/bin:${PATH}"

RUN cargo build --release
