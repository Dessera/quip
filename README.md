# Quip

Quip is a simple chat protocol, which is developing now.

Quip is not a safe protocol because it dose not implements a "REAL"
authentication. Basically, it's just a demo protocol only for learning purpose.

## Critical Unsafe Issues

- `Login` is just an alias of `SetName`, there is not authentication.
- Users can send messages to offline users without restrictions.

## About SSL/TLS

Because there is no way to split a `TlsStream` for full duplex mode in
`tokio_native_tls`, the implementation of `server/listener/tls.rs` should
accepts two sockets (one for read, another for write).

## Roadmap

- [x] Basic commands (`Login`/`Logout`/`Send` etc.)
- [x] Simple doc comments
- [ ] Group commands (`GroupCreate`, `GroupAdd`, `GroupRemove`, `GroupDestroy`)
- [x] Response/Request parser
- [ ] Unit tests
- [x] SSL/TLS
- [ ] Split lib and bins
