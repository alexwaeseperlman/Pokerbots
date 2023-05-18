FROM rust:alpine3.17 as builder

WORKDIR /usr/src/worker

COPY worker/Cargo.toml .
COPY worker/Cargo.lock .

RUN apk add  musl-dev

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY worker .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

FROM alpine:3.17
RUN apk add --update nodejs python3 curl

WORKDIR /usr/src/worker
COPY --from=builder /usr/src/worker/target/release/pokergame /usr/local/bin/pokergame

#CMD ls -la 
RUN chmod +x /usr/local/bin/pokergame
ENTRYPOINT ["pokergame"]