FROM --platform=linux/amd64 rust:alpine3.17 as builder

WORKDIR /usr/src/game

COPY game/Cargo.toml .
COPY game/Cargo.lock .

RUN apk add  musl-dev

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY game .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

FROM --platform=linux/amd64 alpine:3.17
RUN apk add --update nodejs python3 curl

WORKDIR /usr/src/game
COPY --from=builder /usr/src/game/target/release/pokerbots-game /usr/local/bin/pokerbots-game

#CMD ls -la 
RUN chmod +x /usr/local/bin/pokerbots-game
ENTRYPOINT ["pokerbots-game"]