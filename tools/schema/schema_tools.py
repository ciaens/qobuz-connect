import json
import re
import sys
from collections import OrderedDict

bundle = json.load(open(sys.argv[2]))
qon = json.load(open(sys.argv[3]))
BM, BE, BALL = bundle["messages"], bundle["enums"], bundle["all_enums"]
QM, QE = qon["messages"], qon["enums"]

OVERRIDES = {}
if len(sys.argv) > 5:
    OVERRIDES = json.load(open(sys.argv[5]))

def pascal(s):
    return s[0].upper() + s[1:]

def snake(s):
    return re.sub(r"(?<!^)(?=[A-Z])", "_", s).lower()

def by_field(name):
    return [s for s, m in BM.items() if any(f["name"] == name for f in m["fields"])]

names = OrderedDict()
envelope = by_field("messageType")[0]
batch = by_field("messages")[0]
names[envelope] = "QConnectMessage"
names[batch] = "QConnectBatch"
names[by_field("jwt")[0]] = "Authenticate"
names[by_field("reconnect")[0]] = "Disconnect"
names[by_field("dests")[0]] = "Payload"
subs = sorted(by_field("channels"), key=lambda s: BM[s]["line"])
names[subs[0]] = "Subscribe"
names[subs[1]] = "Unsubscribe"
QCLOUD = {names[s] for s in list(names)[2:]}

SPECIAL = {"error": "Error", "playbackError": "PlaybackError", "authenticate": "AuthenticateMessage"}
for f in BM[envelope]["fields"]:
    if f["kind"] == "message":
        names[f["ref"]] = SPECIAL.get(f["name"], pascal(f["name"]))

names.update({k: v for k, v in OVERRIDES.items() if k != "__enums__"})
conflicts = []
queue = list(names)
while queue:
    sym = queue.pop(0)
    qname = names[sym]
    qfields = {f["number"]: f for f in QM.get(qname, {"fields": []})["fields"]}
    for f in BM[sym]["fields"]:
        if f["kind"] != "message":
            continue
        ref = f["ref"]
        qf = qfields.get(f["number"])
        candidate = qf["type"] if qf and qf["kind"] == "message" else pascal(f["name"])
        candidate = OVERRIDES.get(ref, candidate)
        if ref in names:
            if names[ref] != candidate and ref not in OVERRIDES and sym != envelope:
                conflicts.append((ref, names[ref], candidate, f"{qname}.{f['name']}"))
            continue
        if candidate in names.values():
            conflicts.append((ref, candidate, "taken", f"{qname}.{f['name']}"))
            candidate = OVERRIDES.get(ref, candidate + "2")
        names[ref] = candidate
        queue.append(ref)

enum_names = {}
for sym, members in BALL.items():
    ms = [n for n, _ in members if n != "UNRECOGNIZED"]
    if not ms:
        continue
    prefix = ms[0]
    for n in ms[1:]:
        while not n.startswith(prefix):
            prefix = prefix[: prefix.rfind("_")] if "_" in prefix else ""
            if not prefix:
                break
    prefix = prefix.rstrip("_")
    enum_names[sym] = "".join(p.capitalize() for p in prefix.split("_")) if prefix else sym
enum_names.update({s: "MessageType" for s, n in enum_names.items() if n == "MessageType"})
enum_names.update(OVERRIDES.get("__enums__", {}))
used_enum_syms = sorted({f["ref"] for m in BM.values() for f in m["fields"] if f["kind"] == "enum"}, key=lambda s: list(BALL).index(s))

def btype(f):
    if f["kind"] == "scalar":
        return f["type"]
    if f["kind"] == "enum":
        return enum_names[f["ref"]]
    return names[f["ref"]]

def blabel(f):
    return "repeated" if f["repeated"] else ("optional" if f["optional"] else "")

# ------------------------------------------------------------------ diff
def diff():
    out = []
    out.append("# Schema differences: play.qobuz.com bundle versus qonductor 0.1.0-alpha.5\n")
    out.append("Bundle line numbers refer to `web-bundle.js` as dumped on 2026-09-14. Field labels: `optional` means explicit presence in the bundle (ts-proto emits `void 0 !==`), blank means a proto3 field without presence (encoded only when non-default).\n")
    if conflicts:
        out.append("## Naming conflicts to resolve\n")
        for c in conflicts:
            out.append(f"- `{c[0]}`: {c[1]} vs {c[2]} (via {c[3]})")
        out.append("")
    out.append("## Message type enum\n")
    bmt = {v: n for n, v in BALL[[s for s, n in enum_names.items() if n == "MessageType"][0]] if n != "UNRECOGNIZED"}
    qmt = {v: n for n, v in QE["QConnectMessageType"]}
    for v in sorted(set(bmt) | set(qmt)):
        if bmt.get(v) != qmt.get(v):
            out.append(f"- {v}: bundle `{bmt.get(v, '-')}`, qonductor `{qmt.get(v, '-')}`")
    out.append("")
    out.append("## Other enums\n")
    for sym in used_enum_syms:
        en = enum_names[sym]
        b = {v: n for n, v in BALL[sym] if n != "UNRECOGNIZED"}
        q = {v: n for n, v in QE.get(en, [])}
        if en not in QE:
            out.append(f"- `{en}`: absent in qonductor (bundle members: {', '.join(f'{n}={v}' for v, n in sorted(b.items()))})")
            continue
        for v in sorted(set(b) | set(q)):
            if b.get(v) != q.get(v):
                out.append(f"- `{en}` {v}: bundle `{b.get(v, '-')}`, qonductor `{q.get(v, '-')}`")
    out.append("")
    bnames = set(names.values())
    out.append("## Messages only in the bundle\n")
    for sym, n in names.items():
        if n not in QM:
            out.append(f"- `{n}` (bundle line {BM[sym]['line']})")
    out.append("")
    out.append("## Messages only in qonductor\n")
    for n in QM:
        if n not in bnames:
            out.append(f"- `{n}` ({QM[n]['file']})")
    out.append("")
    out.append("## Field differences\n")
    for sym, n in names.items():
        if n not in QM:
            continue
        bf = {f["number"]: f for f in BM[sym]["fields"]}
        qf = {f["number"]: f for f in QM[n]["fields"]}
        rows = []
        for num in sorted(set(bf) | set(qf)):
            b, q = bf.get(num), qf.get(num)
            if b and not q:
                rows.append(f"  - {num}: only in bundle: `{blabel(b)} {btype(b)} {snake(b['name'])}`")
            elif q and not b:
                rows.append(f"  - {num}: only in qonductor: `{q['label']} {q['type']} {q['name']}`")
            else:
                issues = []
                if snake(b["name"]) != q["name"]:
                    issues.append(f"name `{snake(b['name'])}` vs `{q['name']}`")
                bt, qt = btype(b), q["type"]
                if bt != qt and not (b["kind"] == "enum" and qt == "int32") and not (b["kind"] == "enum" and qt == bt):
                    issues.append(f"type `{bt}` vs `{qt}`")
                if b["repeated"] != (q["label"] == "repeated"):
                    issues.append(f"label `{blabel(b)}` vs `{q['label']}`")
                if issues:
                    rows.append(f"  - {num} `{snake(b['name'])}`: " + "; ".join(issues))
        if rows:
            out.append(f"- `{n}` (bundle line {BM[sym]['line']})")
            out.extend(rows)
    out.append("")
    out.append("## Presence semantics\n")
    out.append("qonductor declares every field `optional` in proto2. The bundle is proto3: only the fields listed as `optional` in the generated proto files carry explicit presence, the rest are encoded only when non-default. Wire compatible for decoding; encoding differs only in that zero values are omitted.\n")
    return "\n".join(out)

# ------------------------------------------------------------------ proto
def enum_block(sym, name, indent=""):
    lines = [f"{indent}enum {name} {{"]
    members = [(n, v) for n, v in BALL[sym] if n != "UNRECOGNIZED"]
    if not any(v == 0 for _, v in members):
        prefix = members[0][0].rsplit("_", 1)[0]
        members.insert(0, (f"{prefix}_UNSPECIFIED", 0))
    for n, v in members:
        lines.append(f"{indent}  {n} = {v};")
    lines.append(f"{indent}}}")
    return "\n".join(lines)

def message_block(sym):
    lines = [f"message {names[sym]} {{"]
    for f in BM[sym]["fields"]:
        label = "" if f["kind"] == "message" and not f["repeated"] else blabel(f)
        lines.append(f"  {label + ' ' if label else ''}{btype(f)} {snake(f['name'])} = {f['number']};")
    lines.append("}")
    return "\n".join(lines)

def gen(outdir):
    qcloud = ["syntax = \"proto3\";", "", "package qcloud;", ""]
    qcloud.append("enum MessageType {\n  MESSAGE_TYPE_UNSPECIFIED = 0;\n" + "\n".join(f"  {n} = {v};" for n, v in BALL["s"]) + "\n}")
    qcloud.append("")
    qcloud.append("enum Proto {\n  PROTO_UNKNOWN = 0;\n  PROTO_QCONNECT = 1;\n}")
    qcloud.append("")
    qcloud.append("enum ChannelType {\n  CHANNEL_TYPE_UNSPECIFIED = 0;\n  CHANNEL_TYPE_DEVICE = 1;\n  CHANNEL_TYPE_BACKEND = 2;\n  CHANNEL_TYPE_SESSION_CONTROLLERS = 3;\n}")
    qcloud.append("")
    for sym in sorted([s for s, n in names.items() if n in QCLOUD], key=lambda s: BM[s]["line"]):
        qcloud.append(message_block(sym))
        qcloud.append("")
    open(f"{outdir}/qcloud.proto", "w").write("\n".join(qcloud).rstrip() + "\n")

    qc = ["syntax = \"proto3\";", "", "package qconnect;", ""]
    for sym in used_enum_syms:
        qc.append(enum_block(sym, enum_names[sym]))
        qc.append("")
    for sym in sorted([s for s, n in names.items() if n not in QCLOUD], key=lambda s: BM[s]["line"]):
        qc.append(message_block(sym))
        qc.append("")
    open(f"{outdir}/qconnect.proto", "w").write("\n".join(qc).rstrip() + "\n")
    print("names:", len(names), "enums:", len(used_enum_syms))

if sys.argv[1] == "diff":
    open(sys.argv[4], "w").write(diff())
    print("conflicts", len(conflicts))
    for c in conflicts:
        print(" ", c)
elif sys.argv[1] == "gen":
    gen(sys.argv[4])
    for c in conflicts:
        print("conflict", c)
