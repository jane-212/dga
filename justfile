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

# install cargo-bundle
install-bundle:
    cargo install cargo-bundle

# build project in release mode for windows
build-windows TARGET: install-bundle
    cargo bundle --release --target {{TARGET}} --format msi

# build project in release mode
build TARGET: install-bundle
    cargo bundle --release --target {{TARGET}}
