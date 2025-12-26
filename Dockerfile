FROM rust

WORKDIR /usr/src/network-simulator
COPY . .

RUN cargo install --path .

CMD ["network-simulator"]
