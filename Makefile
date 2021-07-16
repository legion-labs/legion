# make takes the first target as the default target, so please keep it at the top
# all the checks used in the CI
checks: check-format check-build check-clippy check-deps

check-env:
	rustup --version
	cargo deny --version
	mdbook --version
	cmake --version
	python3 --version
	ninja --version

check-format:
	cargo fmt --all -- --check

check-build:
	cargo check --locked --all-targets --all-features

check-clippy:
	cargo clippy --locked --all-targets --all-features -- -D warnings

check-deps:
	cargo deny check

tests: tests-build tests-run

tests-build:
	cargo test --no-run

tests-run:
	cargo test

bench: bench-build bench-run

bench-build:
	cargo bench --no-run

bench-run:
	cargo bench

build-all: build build-release

build:
	cargo build

build-release:
	cargo build --release

docs:
	cargo doc --workspace --no-deps --all-features
	mdbook build ./docs/
	echo "<meta http-equiv=\"refresh\" content=\"0; URL=book/index.html\"/>" > target/doc/index.html

clean:
	cargo clean

.PHONY: check-format check-build check-clippy check-deps check-env tests tests-build tests-run bench bench-build bench-run build-all build build-release docs clean
