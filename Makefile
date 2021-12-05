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
check: check-format check-build check-clippy check-deps

check-env:
	rustup --version
	cargo --version
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

test: test-build test-run

test-build:
	cargo test --no-run
	cargo build -p compiler-*

test-run:
	cargo test -- --skip gpu_

test-gpu-run:
	cargo test -- gpu_

bench: bench-build bench-run

bench-build:
	cargo bench --no-run

bench-run:
	cargo bench

build-all: build build-release

build:
	cargo build

codegen:
	cargo build --features=run-codegen

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
	echo "<meta http-equiv=\"refresh\" content=\"0; URL=legion_app/index.html\"/>" > target/doc/index.html

book:
	mdbook build ./doc/

clean:
	cargo clean

git-clean:
	git clean -fxd

.PHONY: check-format check-build check-clippy check-deps check-env test test-build test-run bench bench-build bench-run build-all build build-release cov grcov timings api-doc book clean git-clean
