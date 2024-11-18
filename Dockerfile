FROM rust:1.82 as builder
WORKDIR	/usr/src
COPY ./src	./src
COPY Cargo.toml	Cargo.lock	./
RUN	cargo build --release

FROM gcr.io/distroless/cc-debian12	as runtime
COPY --from=builder /usr/src/target/release/blockfrost-platform	/bin
EXPOSE 3000/tcp
STOPSIGNAL SIGINT
ENTRYPOINT ["blockfrost-platform"]
