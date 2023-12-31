```bash
sudo apt update
sudo apt upgrade
sudo apt install build-essential libssl-dev pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

```bash
git clone https://github.com/chris13524/relay-subscribe-publish-race-condition.git
```

```bash
export PROJECT_ID=xxx
```

Run server in 1 region:

```bash
cargo run --example server
```

Run client in a different region:

```bash
cargo run --example client
```
