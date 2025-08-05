all: build

build:
	cargo build --release --verbose

clean:
	cargo clean --verbose
	sudo rm -fv /usr/bin/hydock

fix:
	cargo fix --all-targets --workspace --allow-dirty --allow-staged
	cargo fix --clippy --all-targets --workspace --allow-dirty --allow-staged

format:
	cargo fmt --all

install: build
	sudo install -D -m755 -v ./target/release/hydock /usr/bin/hydock
	mkdir -p $(HOME)/.config/hydock/
	cp -fv ./assets/config.toml $(HOME)/.config/hydock/config.toml
	cp -fv ./assets/style.css $(HOME)/.config/hydock/style.css

lint:
	cargo clippy --all-targets --all-features --workspace -- \
		-D warnings \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::unwrap_used \
		-D clippy::expect_used \
		-D clippy::todo \
		-D clippy::dbg_macro \
		-D clippy::panic \
		-D clippy::panic_in_result_fn \
		-D clippy::unwrap_in_result \
		-D clippy::clone_on_ref_ptr \
		-D clippy::large_types_passed_by_value

test:
	cargo test

uninstall:
	sudo rm -fv /usr/bin/hydock
	rm -frv $(HOME)/.config/hydock/

.PHONY: all build clean fix format install lint test uninstall
