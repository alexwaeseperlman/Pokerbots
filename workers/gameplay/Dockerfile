FROM --platform=linux/arm64 rust:1.70 as builder

WORKDIR /usr/src/gameplay

COPY shared /usr/shared

COPY workers/gameplay/Cargo.toml .
#COPY workers/gameplay/Cargo.lock .

# create a dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {println!(\"wawaweewa\")}" > src/main.rs
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN cargo build --release && rm -rf src
COPY workers/gameplay .
# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release 

# Install python3 and pip so people can use common packages in their bots
FROM --platform=linux/arm64 debian:bullseye
RUN apt-get update && apt-get install -y python3 pip ca-certificates

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

# Install common python packages
RUN pip install numpy pandas scipy scikit-learn
RUN apt-get install -y iptables unzip bubblewrap

WORKDIR /usr/src/gameplay
COPY --from=builder /usr/src/gameplay/target/release/gameplay /usr/local/bin/gameplay

RUN adduser runner
RUN chmod +x /usr/local/bin/gameplay
ENTRYPOINT ["gameplay"]
