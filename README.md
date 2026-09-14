# qobuz-connect

Qobuz Connect protocol in Rust, for devices that are controlled by the Qobuz apps (renderers) and for devices that control other renderers (controllers).

Under construction. The crate currently contains the protocol schema with its generated message types and the transport to the Qobuz cloud; the session layers follow.

## Transport

`Transport::connect` opens the WebSocket, authenticates, subscribes to Qobuz Connect and keeps the connection alive, reconnecting with the same schedule as the official web player. Messages go out with `send` and arrive through `recv`, together with `Disconnected` and `Reconnected` events so a session can join again.

`examples/listen.rs` joins a session as a controller renderer and prints every message the cloud sends. It takes the socket endpoint and token from `QOBUZ_CONNECT_ENDPOINT` and `QOBUZ_CONNECT_JWT`; both come from the `qws/createToken` response of the Qobuz API, visible in the browser network tab when play.qobuz.com starts. `examples/decode.rs` pretty-prints a captured WebSocket message given as hex on stdin.

## Schema

`proto/qcloud.proto` describes the outer frames of the Qobuz cloud socket (authenticate, subscribe, payload, disconnect). `proto/qconnect.proto` describes the Qobuz Connect messages carried inside payload frames. Both were lifted from the official web player, which ships the generated encoders for every message, so field numbers and enum values match what the Qobuz apps send. `docs/schema-diff.md` in the repository lists where this schema departs from the qonductor crate.

The Rust types in `src/proto/` are prost output committed to the repository. Regenerate them after editing a proto file:

```
just regenerate
```

This needs no system protoc; the tool under `tools/regen` compiles the schema with protox.

## License

MIT, see `LICENSE`.
