# Build frontend in one container, backend in another, then copy the results into a third container
FROM --platform=linux/arm64 node:20-alpine3.17 as frontend
WORKDIR /usr/src/app
COPY app/package.json package.json
COPY app/package-lock.json ./package-lock.json
RUN npm ci

COPY app .
RUN npm run build

FROM --platform=linux/arm64 rust:1.69 as backend
WORKDIR /usr/src/backend
RUN apt-get update && apt-get install pkg-config libssl-dev libpq-dev curl build-essential -y

# Copy dependency information first so we can cache the build
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
COPY backend/Cargo.toml .
COPY backend/Cargo.lock .

RUN mkdir src && echo 'fn main() { println!("!"); }' > src/main.rs 

RUN cargo build --release && rm -rf src
COPY backend .

# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release

# Copy files and build
FROM --platform=linux/arm64 debian:stable

WORKDIR /usr/src/website

RUN mkdir app
WORKDIR /usr/src/website
COPY --from=backend /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
RUN apt-get update && apt-get install libssl-dev libpq-dev -y

COPY --from=frontend /usr/src/app/dist ./app/dist
COPY --from=backend /usr/src/backend/target/release/pokerbots ./pokerbots

CMD ["./pokerbots"]

