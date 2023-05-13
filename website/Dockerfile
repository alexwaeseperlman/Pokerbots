# Copy files and build
FROM --platform=linux/arm64 debian:stable


#RUN apt-get install nodejs npm
RUN apt-get update  && apt-get install pkg-config libssl-dev libpq-dev curl build-essential -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH=/root/.cargo/bin:$PATH
WORKDIR /usr/src/backend


COPY backend .
COPY app ./app

ENV NODE_VERSION=20.0.0
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

RUN cd app && npm ci
RUN cd app && npm run build
RUN /root/.cargo/bin/cargo install --path . 
CMD ["pokerbots"]

