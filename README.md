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

```bash
while cargo run; do echo "success"; done
```

Server:

```bash
cargo run --example server
```

Client:

```bash
cargo run --example client
```