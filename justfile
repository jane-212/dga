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

# install dependencies
install-dependencies:
    if [ "{{os()}}" == "linux" ]; then \
        sudo apt-get update; \
        sudo apt-get -y install libgtk-3-dev; \
        sudo apt-get -y install libsoup-3.0-dev; \
        sudo apt-get -y install javascriptcoregtk-4.1-dev; \
        sudo apt-get -y install libwebkit2gtk-4.1-dev; \
        sudo apt-get -y install libxkbcommon-x11-dev; \
    fi

# install cargo-bundle
install-bundle:
    cargo install cargo-bundle

flag := if os_family() == "windows" { "--format msi" } else { "" }

# build project in release mode
build TARGET: install-dependencies install-bundle
    cargo bundle --release --target {{TARGET}} {{flag}}
