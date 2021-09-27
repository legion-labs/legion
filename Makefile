# make takes the first target as the default target, so please keep it at the top
# all the checks used in the CI
check: check-format check-build check-clippy check-deps check-dockerize

init:
	yarn install

check-env:
	rustup --version
	cargo deny --version
	mdbook --version
	cmake --version
	python3 --version
	ninja --version
	yarn --version

check-format:
	cargo fmt --all -- --check

check-build:
	cargo check --locked --all-targets --all-features

check-clippy:
	cargo clippy --locked --all-targets --all-features -- -D warnings

check-deps:
	cargo deny check

check-dockerize:
	echo "cargo dockerize check"

test: test-build test-run

test-build:
	cargo test --no-run

test-run:
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

doc:
	cargo doc --workspace --no-deps --all-features
	mdbook build ./doc/
	echo "<meta http-equiv=\"refresh\" content=\"0; URL=book/index.html\"/>" > target/doc/index.html

dockerize:
	echo "cargo dockerize build"

dockerize-release:
	echo "cargo dockerize build --release"

dockerize-push:
	echo "cargo dockerize push --provider=aws"

clean:
	cargo clean

.PHONY: check-format check-build check-clippy check-deps check-env check-dockerize test test-build test-run bench bench-build bench-run build-all build build-release doc dockerize dockerize-deploy clean
