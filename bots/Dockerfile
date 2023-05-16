FROM --platform=linux/amd64 rust:alpine3.17 as builder

WORKDIR /usr/src/game

COPY game/Cargo.toml .
COPY game/Cargo.lock .

RUN apk add libressl-dev musl-dev

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src && rm -rf target/release/deps/pokerbots_game*
COPY game .
RUN cargo build --release 

FROM --platform=linux/amd64 alpine:3.17
RUN apk add --update nodejs python3

COPY --from=builder /usr/src/game/target/release/pokerbots-game /usr/local/bin/pokerbots-game

ENTRYPOINT ["pokerbots-game"]
#CMD ["pokerbots-game"]