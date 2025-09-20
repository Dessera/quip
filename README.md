# Quip

Quip is a simple chat protocol, which is developing now.

Quip is not a safe protocol because it dose not implements a "REAL" authentication. Basically, it's just a demo protocol only for learning purpose.

## Critical Unsafe Issues

- `Login` is just an alias of `SetName`, there is not authentication.
- Users can send messages to offline users without restrictions.
- SSL/TLS support is *IMPOSSIBLE*

## About SSL/TLS

Because there is no way to split a `TlsStream` for full duplex mode, the
implementation of `listener/tls.rs` is a fake implementation, which uses a
`Mutex` (`tokio::io::split`) to ensure thread safe.

I have no way to handle this except rewrite the logic of tls socket (which
means tons of work), so I just leave the unstable one.

Except that, because of the runtime-dependent socket, all interface of `QuipStream`
should be wrap into a `Box`, which may cause some **performance issue**.

I should improve my rust skills :(

## Roadmap

- [x] Basic commands (`Login`/`Logout`/`Send` etc.)
- [x] Simple doc comments
- [ ] Group commands (`GroupCreate`, `GroupAdd`, `GroupRemove`, `GroupDestroy`)
- [ ] Response/Request parser
- [ ] Unit tests
- [x] SSL/TLS
- [ ] Split lib and bins
