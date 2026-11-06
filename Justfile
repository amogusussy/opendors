alias b := build
alias i := install

name := "dors"
path := "./target/release/" + name
bin_path := "/usr/bin/" + name
pam_path := "/etc/pam.d/" + name

build:
    export RUST_BACKTRACE=1
    cargo build --release

install: build
    sudo cp ./pam.conf {{pam_path}}
    sudo cp ./dors.toml /etc/dors.toml
    sudo cp {{path}} {{bin_path}}
    sudo chown root:root {{bin_path}}
    sudo chmod u+s {{bin_path}}
