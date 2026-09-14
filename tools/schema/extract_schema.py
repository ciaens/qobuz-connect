import json
import re
import sys
from collections import defaultdict

BUNDLE = sys.argv[1]
LO, HI = 274000, 286000

with open(BUNDLE, encoding="utf-8", errors="replace") as f:
    lines = f.readlines()

block = lines[LO:HI]
text = "".join(lines)

def norm(s):
    return re.sub(r"\s+", " ", s)

# --- enums ---------------------------------------------------------------
enum_re = re.compile(
    r"!?function\(e\) \{((?:\s*e\[e\.[A-Z0-9_]+ = -?\d+\] = \"[A-Z0-9_]+\",?)+)\s*\}\((\w+) \|\| \(\2 = \{\}\)\)"
)
enums = {}
for m in enum_re.finditer(text):
    members = re.findall(r"e\[e\.([A-Z0-9_]+) = (-?\d+)\]", m.group(1))
    enums[m.group(2)] = [(n, int(v)) for n, v in members]

# enum-to-json helper functions: function Xx(e) { switch (e) { case Var.NAME:
fn_re = re.compile(r"function (\w+)\(e\) \{\s*switch \(e\) \{\s*case (\w+)\.[A-Z0-9_]+:")
fn_to_enum = {}
for m in fn_re.finditer(text):
    if m.group(2) in enums:
        fn_to_enum[m.group(1)] = m.group(2)

# --- messages ------------------------------------------------------------
msgs = {}
i = 0
while i < len(block):
    m = re.match(r"^\s*(?:const |var |let |, )?([\w$]+) = \{\s*$", block[i])
    if m and i + 1 < len(block) and block[i + 1].lstrip().startswith("encode"):
        sym = m.group(1)
        start = LO + i
        j = i + 1
        while not re.match(r"^\s*decode\(e, t\) \{", block[j]):
            j += 1
        encode_body = norm("".join(block[i + 1 : j])).replace("return ", "")
        k = j
        while not re.match(r"^\s*toJSON\(e\) \{", block[k]):
            k += 1
        decode_body = norm("".join(block[j:k]))
        depth = 0
        e = k
        while True:
            depth += block[e].count("{") - block[e].count("}")
            e += 1
            if depth <= 0 and e > k + 1:
                break
        tojson_body = norm("".join(block[k:e]))
        msgs[sym] = dict(line=start + 1, encode=encode_body, decode=decode_body, tojson=tojson_body)
        i = e
    else:
        i += 1

FIELD_PATTERNS = [
    ("repeated_message", re.compile(r"for \(const r of e\.(\w+)\) ([\w$]+)\.encode\(r, t\.uint32\((\d+)\)\.fork\(\)\)\.ldelim\(\)")),
    ("repeated_packed", re.compile(r"t\.uint32\((\d+)\)\.fork\(\); for \(const r of e\.(\w+)\) t\.(\w+)\(r\); t\.ldelim\(\)")),
    ("repeated_scalar", re.compile(r"for \(const r of e\.(\w+)\) t\.uint32\((\d+)\)\.(\w+)\(r\)")),
    ("message", re.compile(r"void 0 !== e\.(\w+) && ([\w$]+)\.encode\(e\.\1, t\.uint32\((\d+)\)\.fork\(\)\)\.ldelim\(\)")),
    ("optional_scalar", re.compile(r"void 0 !== e\.(\w+) && t\.uint32\((\d+)\)\.(\w+)\(e\.\1\)")),
    ("scalar", re.compile(r"(?:\"\" !== |0 !== |!1 !== |!0 !== |!0 === |!1 === )e\.(\w+)(?:\.length)? && t\.uint32\((\d+)\)\.(\w+)\(e\.\1\)")),
]

def parse_fields(sym, body):
    fields = []
    consumed = body
    for kind, rx in FIELD_PATTERNS:
        for m in rx.finditer(body):
            if kind == "repeated_message":
                name, ref, tag = m.group(1), m.group(2), int(m.group(3))
                fields.append(dict(name=name, number=tag >> 3, kind="message", ref=ref, repeated=True, optional=False))
            elif kind == "repeated_packed":
                tag, name, typ = int(m.group(1)), m.group(2), m.group(3)
                fields.append(dict(name=name, number=tag >> 3, kind="scalar", type=typ, repeated=True, optional=False))
            elif kind == "repeated_scalar":
                name, tag, typ = m.group(1), int(m.group(2)), m.group(3)
                fields.append(dict(name=name, number=tag >> 3, kind="scalar", type=typ, repeated=True, optional=False))
            elif kind == "message":
                name, ref, tag = m.group(1), m.group(2), int(m.group(3))
                fields.append(dict(name=name, number=tag >> 3, kind="message", ref=ref, repeated=False, optional=True))
            elif kind == "optional_scalar":
                name, tag, typ = m.group(1), int(m.group(2)), m.group(3)
                fields.append(dict(name=name, number=tag >> 3, kind="scalar", type=typ, repeated=False, optional=True))
            else:
                name, tag, typ = m.group(1), int(m.group(2)), m.group(3)
                fields.append(dict(name=name, number=tag >> 3, kind="scalar", type=typ, repeated=False, optional=False))
            consumed = consumed.replace(m.group(0), "", 1)
    leftover = re.sub(r"encode(?:: \(e, t=D\.Writer\.create\(\)\) => \(|\(e, t=D\.Writer\.create\(\)\) \{)", "", consumed)
    leftover = re.sub(r"return t|[\s,;(){}]+", " ", leftover).strip()
    leftover = re.sub(r"^t$", "", leftover)
    seen = {}
    for f in fields:
        seen[f["number"]] = f
    return [seen[n] for n in sorted(seen)], leftover

def enum_fields(tojson):
    out = {}
    for m in re.finditer(r"t\.(\w+) = (?:void 0 !== e\.\1 \? )?(\w+)\(e\.\1\)", tojson):
        if m.group(2) in fn_to_enum:
            out[m.group(1)] = fn_to_enum[m.group(2)]
    for m in re.finditer(r"t\.(\w+) = function\(e\) \{ switch \(e\) \{ case (\w+)\.", tojson):
        if m.group(2) in enums:
            out[m.group(1)] = m.group(2)
    for m in re.finditer(r"t\.(\w+) = e\.\1\.map\(\(?e\)? => (\w+)\(e\)\)", tojson):
        if m.group(2) in fn_to_enum:
            out[m.group(1)] = fn_to_enum[m.group(2)]
    return out

result = {}
for sym, m in msgs.items():
    fields, leftover = parse_fields(sym, m["encode"])
    ef = enum_fields(m["tojson"])
    for f in fields:
        if f["name"] in ef:
            f["kind"] = "enum"
            f["ref"] = ef[f["name"]]
    result[sym] = dict(line=m["line"], fields=fields, leftover=leftover, encode=m["encode"])

parents = defaultdict(set)
for sym, m in result.items():
    for f in m["fields"]:
        if f["kind"] == "message":
            parents[f["ref"]].add(f"{sym}.{f['name']}")

used_enums = {f["ref"] for m in result.values() for f in m["fields"] if f["kind"] == "enum"}

out = dict(
    messages=result,
    parents={k: sorted(v) for k, v in parents.items()},
    enums={k: v for k, v in enums.items() if k in used_enums},
    all_enums=enums,
    unresolved_enum_fns=sorted(set(fn_to_enum) - set()),
)
json.dump(out, open(sys.argv[2], "w"), indent=1)

print("messages", len(result))
print("enums used", len(out["enums"]), "of", len(enums))
bad = {s: m["leftover"] for s, m in result.items() if m["leftover"]}
print("messages with unparsed encode statements", len(bad))
for s, l in list(bad.items())[:12]:
    print(" ", s, "line", result[s]["line"], "->", l[:160])
