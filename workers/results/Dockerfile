FROM --platform=linux/arm64 rust:1.70 as builder

WORKDIR /usr/src/results

COPY shared /usr/shared

COPY workers/results/Cargo.toml .

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY workers/results .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

# Install python3 and pip so people can use common packages in their bots
FROM --platform=linux/arm64 debian:bullseye

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

WORKDIR /usr/src/results
COPY --from=builder /usr/src/results/target/release/results /usr/local/bin/results
RUN apt-get update && apt-get install libssl-dev libpq-dev ca-certificates -y

RUN chmod +x /usr/local/bin/results
ENTRYPOINT ["results"]