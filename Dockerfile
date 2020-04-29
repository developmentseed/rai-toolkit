FROM alpine:3.11

ENV HOME=/home/rai
COPY ./ $HOME/toolkit
WORKDIR $HOME/toolkit

RUN apk add curl gcc
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 0.43 | sh -s -- -y

ENV PATH="$HOME/.cargo/bin:${PATH}"

RUN cargo build --release
