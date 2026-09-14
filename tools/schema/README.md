# Schema extraction

The protobuf schema in `proto/` is lifted from the play.qobuz.com web player bundle, which ships ts-proto generated encoders for every Qobuz Connect message. These scripts reproduce `proto/*.proto` and `docs/schema-diff.md` from a fresh bundle dump.

```
python3 extract_schema.py <bundle.js> schema.json
python3 parse_proto.py qonductor.json <qonductor>/proto/qconnect_{envelope,common,queue,payload}.proto
python3 schema_tools.py diff schema.json qonductor.json ../../docs/schema-diff.md overrides.json
python3 schema_tools.py gen schema.json qonductor.json ../../proto overrides.json
```

`extract_schema.py` scans the bundle for message objects with `encode` and `decode` members, reads field numbers, wire types, presence and repetition from each `encode` body, and resolves enum-typed fields through the `toJSON` helpers. `overrides.json` fixes the names of nested messages that the envelope does not name and renames the message type enum so prost strips the `MESSAGE_TYPE_` prefix.

Regenerate the Rust code in `src/proto/` with prost-build and protox (no system protoc needed):

```
just regenerate
```
