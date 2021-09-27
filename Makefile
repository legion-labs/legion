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

check-format: init
	cargo fmt --all -- --check

check-build: init
	cargo check --locked --all-targets --all-features

check-clippy: init
	cargo clippy --locked --all-targets --all-features -- -D warnings

check-deps: init
	cargo deny check

check-dockerize: init
	echo "cargo dockerize check"

test: test-build test-run

test-build: init
	cargo test --no-run

test-run: init
	cargo test

bench: bench-build bench-run

bench-build: init
	cargo bench --no-run

bench-run: init
	cargo bench

build-all: build build-release

build: init
	cargo build

build-release: init
	cargo build --release

doc: init
	cargo doc --workspace --no-deps --all-features
	mdbook build ./doc/
	echo "<meta http-equiv=\"refresh\" content=\"0; URL=book/index.html\"/>" > target/doc/index.html

dockerize: init
	echo "cargo dockerize build"

dockerize-release: init
	echo "cargo dockerize build --release"

dockerize-push: init
	echo "cargo dockerize push --provider=aws"

clean:
	cargo clean

.PHONY: init check-format check-build check-clippy check-deps check-env check-dockerize test test-build test-run bench bench-build bench-run build-all build build-release doc dockerize dockerize-deploy clean
