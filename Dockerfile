	
FROM	rust:1.82	as	builder
WORKDIR	/usr/src
COPY	Cargo.toml	Cargo.lock	./
RUN	mkdir src \
	&& touch src/lib.rs
RUN	cargo build --release \
	&& rm -rf src
COPY	./src	./src
RUN	cargo build --release
FROM	rust:1.82	as	runtime
COPY	/usr/src/target/release/blockfrost-platform	/usr/local/bin/blockfrost-platform
EXPOSE	3000/tcp
STOPSIGNAL	SIGINT
ENTRYPOINT	["/usr/local/bin/blockfrost-platform"]
