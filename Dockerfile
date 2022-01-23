FROM rust:alpine as server
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev openssl-dev
COPY . .
RUN cargo test --bin tanks_server --release
RUN cargo build --bin tanks_server --release

FROM rust:alpine as wasm
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev openssl-dev
# install wasm-bindgen
RUN cargo install -f wasm-bindgen-cli
# install WASM target
RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN cargo run --bin pack-wasm

FROM scratch as deployment
COPY --from=wasm /home/rust/src/dist dist
COPY --from=server /home/rust/src/target/release/tanks_server .
ENV RUST_LOG=info
ENTRYPOINT [ "./tanks_server" ]