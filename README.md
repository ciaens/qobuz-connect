# qobuz-connect

Qobuz Connect protocol in Rust, for devices that are controlled by the Qobuz apps (renderers) and for devices that control other renderers (controllers).

Under construction. The crate currently contains the protocol schema with its generated message types, the transport to the Qobuz cloud, and the session layer with the renderer role; the controller role follows.

## Session

`Session::join` connects, joins the account's session as a controller renderer like the official apps do, and keeps the membership across reconnects. `recv` delivers typed events: registration with the renderer id, commands for this renderer (set state, volume, mute, activation, quality, loop and shuffle mode), session state, changes of the other renderers, queue changes, and errors. A set-state command is a delta: fields it leaves out are unchanged, a track means jump to it, a bare position is a seek, and a bare playing state is play or pause. Always report the track actually playing; a renderer that reports none while playing gets paused by the server. `report` sends this renderer's state, volume, mute, audio quality, device info, user actions and playback errors, stamped with the queue version the session tracks. `activate` makes the device the session's active renderer.

Controllers extrapolate a renderer's position from the timestamp of its last state report, so report state whenever it changes and repeat it at most every few seconds while playing; a stale position sent again would make positions jump.

`examples/fake_renderer.rs` joins with the credentials from `QOBUZ_CONNECT_ENDPOINT` and `QOBUZ_CONNECT_JWT`, prints every event, obeys commands with a simulated position, and activates itself when `QOBUZ_CONNECT_ACTIVATE` is set. Pick it in a Qobuz app to drive it.

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
