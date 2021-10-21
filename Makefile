# Code coverage needs special flags that can't be set somewhere else
ifeq ($(MAKECMDGOALS),cov)
export RUSTC_BOOTSTRAP=1
export RUSTFLAGS=-Zinstrument-coverage
export RUSTDOCFLAGS=-Zinstrument-coverage -Zunstable-options --persist-doctests $(abspath target/debug/doc_bins)
export LLVM_PROFILE_FILE=legion-%p-%m.profraw
endif

ifeq ($(MAKECMDGOALS),timings)
export RUSTC_BOOTSTRAP=1
endif


# make takes the first target as the default target, so please keep it at the top
# all the checks used in the CI
check: check-format check-build check-clippy check-deps check-dockerize

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

cov:
	cargo clean
	cargo build
	cargo test --no-run
	cargo test

grcov:
	grcov . \
	--binary-path ./target/debug/ \
	--source-dir . \
	--output-type html \
	--branch \
	--ignore-not-existing \
	--output-path ./target/debug/coverage/
	find . -name "*.profraw" -type f -delete

timings:
	rm -rf timings
	mkdir timings
	echo "<html><head><title>Cargo Build Timings</title></head><body><h1>Build Timings</h1>" > timings/index.html 
	for TARGET in runtime-srv editor-srv editor-client ; do \
		cargo clean && \
		cargo build --bin $$TARGET -Z timings=html && \
		mv cargo-timing.html timings/$$TARGET.html && \
		echo "<h3><a href=\"./$$TARGET.html\"> * $$TARGET </a></h3>" >> timings/index.html; \
		cargo build --bin $$TARGET --release -Z timings=html && \
		mv cargo-timing.html timings/$$TARGET-release.html && \
		echo "<h3><a href=\"./$$TARGET-release.html\"> * $$TARGET - Release </a></h3>" >> timings/index.html ;\
	done
	rm cargo-timing-*
	echo "</body></html>" >> timings/index.html 

api-doc:
	cargo doc --workspace --no-deps --all-features

book:
	mdbook build ./doc/

dockerize:
	echo "cargo dockerize build"

dockerize-release:
	echo "cargo dockerize build --release"

dockerize-push:
	echo "cargo dockerize push --provider=aws"

clean:
	cargo clean

.PHONY: check-format check-build check-clippy check-deps check-env check-dockerize test test-build test-run bench bench-build bench-run build-all build build-release cov grcov timings api-doc book dockerize dockerize-deploy clean
