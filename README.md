# qobuz-connect

Qobuz Connect protocol in Rust, for devices that are controlled by the Qobuz apps (renderers) and for devices that control other renderers (controllers).

Under construction. The crate contains the protocol schema with its generated message types, the transport to the Qobuz cloud, the session layer with the renderer and controller roles, and the token helper. A device joined through the cloud appears in the device picker of the Qobuz apps, so the crate has no LAN discovery; the LAN handshake of the native apps only serves devices that hold no Qobuz credentials of their own.

## Session

`Session::join` connects, joins the account's session as a controller renderer like the official apps do, and keeps the membership across reconnects. `recv` delivers typed events: registration with the renderer id, commands for this renderer (set state, volume, mute, activation, quality, loop and shuffle mode), session state, changes of the other renderers, queue changes, and errors. A set-state command is a delta: fields it leaves out are unchanged, a track means jump to it, a bare position is a seek, and a bare playing state is play or pause. Always report the track actually playing; a renderer that reports none while playing gets paused by the server. `report` sends this renderer's state, volume, mute, audio quality, device info, user actions and playback errors, stamped with the queue version the session tracks. `activate` makes the device the session's active renderer.

Controllers extrapolate a renderer's position from the timestamp of its last state report, so report state whenever it changes and repeat it at most every few seconds while playing; a stale position sent again would make positions jump.

`examples/fake_renderer.rs` joins with the credentials from `QOBUZ_CONNECT_ENDPOINT` and `QOBUZ_CONNECT_JWT`, prints every event, obeys commands with a simulated position, and activates itself when `QOBUZ_CONNECT_ACTIVATE` is set. Pick it in a Qobuz app to drive it.

## Controller

`control` sends a controller command: the player state of the active renderer (a delta like the one a renderer receives: a queue item means jump to it, a bare position is a seek, a bare playing state is play or pause), the active renderer, volume, mute, maximum quality and loop mode of a renderer, and the queue changes: clear, load, insert, add, remove, reorder, shuffle and autoplay modes, autoplay tracks, or the whole queue state. Commands go out in order. A queue change carries the queue version the session tracks and an action uuid, which `control` returns; the commands behind it wait until the server answers with the queue event that echoes that uuid and carries the new version and, for loads, inserts and adds, the queue item ids of the new tracks. A queue error drops the commands still waiting and asks for the queue state again, as the official apps do.

`examples/controller.rs` joins with the same credentials, makes the renderer whose name contains `QOBUZ_CONNECT_RENDERER` (default `fake renderer`) active, loads the track ids listed in `QOBUZ_CONNECT_TRACKS`, then plays, pauses, seeks, resumes, skips to the second track and pauses, five seconds apart. Run it against `fake_renderer` or against the web player to watch the commands reach a real renderer.

## Transport

`Transport::connect` opens the WebSocket, authenticates, subscribes to Qobuz Connect and keeps the connection alive, reconnecting with the same schedule as the official web player. Messages go out with `send` and arrive through `recv`, together with `Disconnected` and `Reconnected` events so a session can join again.

`examples/listen.rs` joins a session as a controller renderer and prints every message the cloud sends. It takes the socket endpoint and token from `QOBUZ_CONNECT_ENDPOINT` and `QOBUZ_CONNECT_JWT`; both come from the `qws/createToken` response of the Qobuz API, visible in the browser network tab when play.qobuz.com starts. `examples/decode.rs` pretty-prints a captured WebSocket message given as hex on stdin.

## Token

The cloud socket authenticates with a Qobuz Connect token that the Qobuz API mints for a logged-in user. `TokenRequest::new` describes the request for any HTTP client, a POST of `jwt=jwt_qws` to `qws/createToken` with the app id and user auth token headers every Qobuz API call carries, and `Credentials::from_json` reads the response. The crate does no HTTP itself.

The cloud accepts one socket per token, and the official apps mint a new token before every connection. `Session::join_with` and `Transport::connect_with` take a closure that mints one, called again before every reconnect. `join` and `connect` reuse the token they were given, which is enough for the examples and for short sessions.

`examples/token.rs` prints the request as a curl command when `QOBUZ_APP_ID` and `QOBUZ_USER_AUTH_TOKEN` are set, both visible on any API request in the browser network tab of play.qobuz.com, and turns a response piped on stdin into the two exports the other examples need:

```
eval "$(cargo run -q --example token | sh | cargo run -q --example token)"
```

Run it once per process, since two processes cannot share a token.

## Schema

`proto/qcloud.proto` describes the outer frames of the Qobuz cloud socket (authenticate, subscribe, payload, disconnect). `proto/qconnect.proto` describes the Qobuz Connect messages carried inside payload frames. Both were lifted from the official web player, which ships the generated encoders for every message, so field numbers and enum values match what the Qobuz apps send. `docs/schema-diff.md` in the repository lists where this schema departs from the qonductor crate.

The Rust types in `src/proto/` are prost output committed to the repository. Regenerate them after editing a proto file:

```
just regenerate
```

This needs no system protoc; the tool under `tools/regen` compiles the schema with protox.

## License

MIT, see `LICENSE`.
