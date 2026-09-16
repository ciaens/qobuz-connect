# qobuz-connect

Qobuz Connect protocol in Rust: renderers that the Qobuz apps control, and controllers that drive other renderers, both joined through the Qobuz cloud like the official apps. A device joined this way appears in the device picker of every Qobuz app, so there is no LAN discovery to do. Schema and behaviour were lifted from the web player; the crate is not affiliated with Qobuz.

## Usage

The cloud socket authenticates with a token the Qobuz API mints for a logged-in user. `TokenRequest::new` describes that request for any HTTP client, the crate does none, and `Credentials::from_json` reads the answer. The cloud serves one socket per token, so join with a closure that mints one before every connection:

```rust
let mut session = Session::join_with(move || mint(client.clone()), device).await?;
while let Some(event) = session.recv().await {
    if let Event::Command(command) = event {
        let state = player.apply(command);
        session.report(RendererReport::State(state))?;
    }
}
```

**Renderer.** `recv` delivers typed events: registration with the renderer id, commands for this renderer, session state, the other renderers, queue changes and errors. A set-state command is a delta: fields left out are unchanged, a track means jump to it, a bare position is a seek, a bare playing state is play or pause, a negative queue item id is stop. `report` sends state, volume, mute, quality, device info, user actions and playback errors, stamped with the queue version the session tracks. Report the track actually playing, whenever the state changes and at most every few seconds while playing: controllers extrapolate the position from the last report, and a renderer that plays without a track gets paused. `activate` makes the device the active renderer.

**Controller.** `control` sends the player state of the active renderer, the same delta, the active renderer, volume, mute, quality and loop mode of a renderer, and the queue changes: clear, load, insert, add, remove, reorder, shuffle and autoplay, or the whole queue state. Commands go out in order. A queue change carries the queue version and an action uuid, which `control` returns; the commands behind it wait for the queue event that echoes the uuid, or ten seconds. A queue error drops what is waiting and asks for the queue state again, as the apps do.

**Connection.** Sends are queued and never block; `recv` is safe to cancel. The transport reconnects with the web player's backoff, minting a token each time, and treats a minute without a frame as a lost connection. `Reconnected` means the join was sent again: the renderer id, the active flag and the waiting commands are gone until the server registers the device again. Logs go through `tracing` in a span named after the device, message types at debug and payloads at trace.

## Examples

All take `QOBUZ_CONNECT_ENDPOINT` and `QOBUZ_CONNECT_JWT`, which `token` produces from `QOBUZ_APP_ID` and `QOBUZ_USER_AUTH_TOKEN`, both visible on any API request in the browser network tab of play.qobuz.com:

```
eval "$(cargo run -q --example token | sh | cargo run -q --example token)"
```

- `fake_renderer` joins, prints every event, obeys commands with a simulated position and activates itself when `QOBUZ_CONNECT_ACTIVATE` is set. Pick it in a Qobuz app.
- `controller` makes the renderer named by `QOBUZ_CONNECT_RENDERER` (default `fake renderer`) active, loads the track ids in `QOBUZ_CONNECT_TRACKS`, then plays, pauses, seeks, resumes and skips, five seconds apart.
- `listen` prints every message the cloud sends; `decode` pretty-prints a captured message given as hex on stdin.

One token per process: two sockets sharing one evict each other.

## Schema

`proto/qcloud.proto` is the outer frame of the cloud socket, `proto/qconnect.proto` the Qobuz Connect messages inside its payloads, both lifted from the web player so field numbers and enum values match the apps. `src/proto/` is committed prost output; `just regenerate` rebuilds it with protox, no system protoc needed. `docs/schema-diff.md` lists where the schema departs from the qonductor crate.

## License

MIT
