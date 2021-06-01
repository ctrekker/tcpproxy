# tcpproxy | An extraordinarily simple TCP connection proxy
## Getting Started
This project is written in Rust and can be easily installed with Cargo:
```
cargo install --git https://github.com/ctrekker/tcpproxy.git
```
From there we will create a proxy which exposes a locally bound socket on port 8080 to a publically bound port 80.
```
tcpproxy -h 0.0.0.0 -p 80 -H 127.0.0.1 -P 8080
```
Or the more readable and verbose form:
```
tcpproxy --host 0.0.0.0 --port 80 --proxy-host 127.0.0.1 --proxy-port 8080
```