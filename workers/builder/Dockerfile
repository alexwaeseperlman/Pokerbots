FROM --platform=linux/arm64 rust:alpine3.17 as builder

WORKDIR /usr/src/builder

COPY shared /usr/shared

COPY workers/builder/Cargo.toml .

RUN apk add musl-dev

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY workers/builder .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

# Install python3 and pip so people can use common packages in their bots
FROM --platform=linux/arm64 ubuntu:22.04
RUN apt-get update && apt-get install -y python3 pip

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Install common python packages
RUN pip install numpy pandas scipy scikit-learn
RUN apt-get install -y iptables

WORKDIR /usr/src/builder
COPY --from=builder /usr/src/builder/target/release/builder /usr/local/bin/builder

RUN chmod +x /usr/local/bin/builder
ENTRYPOINT ["builder"]