FROM --platform=linux/arm64 rust:alpine3.17 as builder

WORKDIR /usr/src/results

COPY shared /usr/shared

COPY workers/results/Cargo.toml .

RUN apk add musl-dev

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY workers/results .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

# Install python3 and pip so people can use common packages in their bots
FROM --platform=linux/arm64 ubuntu:22.04

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

WORKDIR /usr/src/results
COPY --from=builder /usr/src/results/target/release/results /usr/local/bin/results

RUN chmod +x /usr/local/bin/results
ENTRYPOINT ["results"]