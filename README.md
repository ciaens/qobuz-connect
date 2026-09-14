# qobuz-connect

Qobuz Connect protocol in Rust, for devices that are controlled by the Qobuz apps (renderers) and for devices that control other renderers (controllers).

Under construction. The current release contains the protocol schema and its generated message types; the transport and session layers follow, see `ROADMAP.md` in the repository.

## Schema

`proto/qcloud.proto` describes the outer frames of the Qobuz cloud socket (authenticate, subscribe, payload, disconnect). `proto/qconnect.proto` describes the Qobuz Connect messages carried inside payload frames. Both were lifted from the official web player, which ships the generated encoders for every message, so field numbers and enum values match what the Qobuz apps send. `docs/schema-diff.md` in the repository lists where this schema departs from the qonductor crate.

The Rust types in `src/proto/` are prost output committed to the repository. Regenerate them after editing a proto file:

```
just regenerate
```

This needs no system protoc; the tool under `tools/regen` compiles the schema with protox.

## License

MIT, see `LICENSE`.
