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

flag := if os_family() == "windows" { "--format msi" } else { "" }

# build project in release mode
build TARGET: install-bundle
    cargo bundle --release --target {{TARGET}} {{flag}}
