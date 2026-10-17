alias b := build
alias i := install

name := "dors"
path := "./target/release/" + name
pam_path := "/etc/pam.d/" + name

build:
    export RUST_BACKTRACE=1
    cargo build --release

install: build
    sudo chown root:root {{path}}
    sudo chmod u+s {{path}}
    sudo cp ./pam.conf {{pam_path}}
    sudo cp ./dors.toml /etc/dors.toml
