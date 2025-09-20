# Tchat

Tchat is a simple chat protocol, which is developing now.

Tchat is not a safe protocol because it dose not implements a "REAL" authentication. Basically, it's just a demo protocol only for learning purpose.

## Critical Unsafe Issues

- `Login` is just an alias of `SetName`, there is not authentication.
- Users can send messages to offline users without restrictions.
- (TODO) The server only supports tcp now.

## Roadmap

- [x] Basic commands (`Login`/`Logout`/`Send` etc.)
- [x] Simple doc comments
- [ ] Group commands (`GroupCreate`, `GroupAdd`, `GroupRemove`, `GroupDestroy`)
- [ ] Response/Request parser
- [ ] Unit tests
- [ ] SSL/TLS
- [ ] Split lib and bins
