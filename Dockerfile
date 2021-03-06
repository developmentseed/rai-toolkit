FROM ubuntu:20.04

EXPOSE 4001
ENV HOME=/home/rai
COPY ./ $HOME/toolkit
WORKDIR $HOME/toolkit
ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && apt-get install -y tzdata \
    && dpkg-reconfigure --frontend noninteractive tzdata \
    && apt-get install -y curl build-essential postgresql postgresql-postgis postgis libssl-dev pkg-config \
    && apt-get install -y gdal-bin jq \
    && echo "local   all     all     trust" > /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     localhost       trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     127.0.0.1/32    trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     ::1/128         trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && echo "host    all     all     0.0.0.0/0       trust" >> /etc/postgresql/12/main/pg_hba.conf \
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs 0.43 | sh -s -- -y

RUN cd ${HOME} \
    && mkdir ./mason \
    && curl -sSfL https://github.com/mapbox/mason/archive/v0.22.0.tar.gz | tar -z --extract --strip-components=1 --exclude="*md" --exclude="test*" --directory="./mason/" \
    && ./mason/mason install osmium-tool 1.12.0 \
    && cp mason_packages/linux-x86_64/osmium-tool/1.12.0/bin/osmium /usr/bin/

RUN curl 'https://nodejs.org/dist/v12.18.1/node-v12.18.1-linux-x64.tar.gz' | tar -xzv \
    && cp ./node-v12.18.1-linux-x64/bin/node /usr/bin/ \
    && ./node-v12.18.1-linux-x64/bin/npm install -g npm \
    && npm install -g yarn \
    && cd web \
    && yarn install \
    && yarn build

ENV PATH="$HOME/.cargo/bin:${PATH}"

RUN cargo build --release \
    && cp ${HOME}/toolkit/target/release/rai-toolkit /usr/bin/

CMD service postgresql start \
    && tail -f /dev/null
