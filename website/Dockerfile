# Copy files and build
FROM --platform=linux/arm64 debian:stable


#RUN apt-get install nodejs npm
RUN apt-get update  && apt-get install pkg-config libssl-dev libpq-dev curl build-essential -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /usr/src/backend


ENV NODE_VERSION=20.0.0
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

# Copy dependency information first so we can cache the build
COPY backend/Cargo.toml .
COPY backend/Cargo.lock .
RUN mkdir app
COPY app/package.json ./app/package.json
COPY app/package-lock.json ./app/package-lock.json

# Create a dummy file to force cargo to build the dependencies
RUN mkdir ./src && echo 'fn main() { println!("!"); }' > ./src/main.rs && echo 'fn no() {}' > ./src/lib.rs && cargo install --path . && rm ./src/main.rs ./src/lib.rs
RUN cd app && npm ci

# Now copy the rest of the source code
COPY app ./app
RUN cd app && npm run build

COPY backend .
RUN /root/.cargo/bin/cargo install --path . 

CMD ["pokerbots"]

