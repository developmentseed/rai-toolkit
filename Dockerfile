FROM ubuntu:20.04

ENV HOME=/home/rai
COPY ./ $HOME/toolkit
WORKDIR $HOME/toolkit
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && apt-get install -y tzdata \
    && dpkg-reconfigure --frontend noninteractive tzdata \
    && apt-get install -y curl build-essential postgresql postgresql-postgis libssl-dev pkg-config \
    && apt-get install -y gdal-bin \
    && echo "local   all     all     trust" > /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     localhost       trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     127.0.0.1/32    trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     ::1/128         trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     0.0.0.0/0       trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 0.43 | sh -s -- -y

ENV PATH="$HOME/.cargo/bin:${PATH}"

RUN cargo build --release \
    && cp ${HOME}/toolkit/target/release/rai-toolkit /usr/bin/

CMD service postgresql start \
    && tail -f /dev/null
