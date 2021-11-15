# clipd

This is a simple distributed clipboard.

Imagine this scenario: you use a a couple of different machines to do your
work. You need a way to share clipboard contents between all these machines.
`clipd` solves this problem for you. Imagine `xclip` but with a server and
clients.

## Build

1. `rustup target add x86_64-unknown-linux-musl`
1. `cargo build --target x86_64-unknown-linux-musl --release`

## Server installation

1. `scp ./target/x86_64-unknown-linux-musl/release/clipd_server root@<server>:/usr/local/bin`
1. `scp ./etc/clipd.service root@<server>:/etc/systemd/system`
1. `ssh root@<server> systemctl enable --now clipd`

## Client installation

1. `cp ./target/x86_64-unknown-linux-musl/release/clipd /usr/local/bin`
1. `mkdir -p ~/.config/clipd`
1. `cp ./etc/client.toml ~/.config/clipd`
1. Modify `~/.config/clipd/client.toml`'s `server` field to point to server
