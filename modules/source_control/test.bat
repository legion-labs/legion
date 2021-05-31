@echo off
rmdir /Q /S d:\temp\data
cargo run -- init-local -d d:\temp\data
