# print help
default:
    just --list

alias r := run
# run project in debug mode
run:
    cargo run

alias rr := release
# run project in release mode
release:
    cargo run --release

# build project in release mode
build TARGET:
    cargo install cargo-bundle
    cargo bundle --release --target {{TARGET}}
