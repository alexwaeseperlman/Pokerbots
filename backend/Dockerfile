# Copy files and build
FROM rust:1.67

WORKDIR /usr/src/backend

COPY . .
RUN cargo install --path .
CMD ["pokerbots"]

