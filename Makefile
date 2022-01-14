.PHONY: all test clean

dev-install:
	cargo install systemfd cargo-watch

clean:
	cargo clean

build:
	cargo build

run:
	cargo run

dev:
	systemfd --no-pid -s http::PORT -- cargo watch -x run

db:
	docker-compose up postgres
