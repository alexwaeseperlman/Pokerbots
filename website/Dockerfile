# Build frontend in one container, backend in another, then copy the results into a third container
FROM --platform=linux/arm64 node:20.2.0-bullseye-slim as frontend
WORKDIR /usr/src/app
COPY website/app/package.json package.json
COPY website/app/package-lock.json ./package-lock.json
RUN npm ci

COPY website/app .
RUN npm run build

FROM --platform=linux/arm64 rust:1.70 as backend
WORKDIR /usr/src/backend
RUN apt-get update && apt-get install pkg-config libssl-dev libpq-dev curl build-essential -y

# Copy dependency information first so we can cache the build
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
COPY website/backend/Cargo.toml .
#COPY website/backend/Cargo.lock .

COPY shared /usr/shared

RUN mkdir src && echo 'fn main() { println!("!"); }' > src/main.rs 

RUN cargo build --release && rm -rf src
COPY website/backend .

# touch the main file to force a rebuild
RUN touch src/main.rs
RUN cargo build --release

# Copy files and build
FROM --platform=linux/arm64 debian:bullseye

WORKDIR /usr/src/website

RUN mkdir app
WORKDIR /usr/src/website

COPY --from=frontend /usr/src/app/dist ./app/dist
COPY --from=backend /usr/src/backend/target/release/upac_web ./upac_web

RUN apt-get update && apt-get install -y libssl-dev ca-certificates libpq-dev openssl

CMD ["./upac_web"]

