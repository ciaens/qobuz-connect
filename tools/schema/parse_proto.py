import json
import re
import sys

SCALARS = {"double", "float", "int32", "int64", "uint32", "uint64", "sint32", "sint64", "fixed32", "fixed64", "sfixed32", "sfixed64", "bool", "string", "bytes"}

messages = {}
enums = {}
for path in sys.argv[2:]:
    src = re.sub(r"//[^\n]*", "", open(path).read())
    stack = []
    for line in src.splitlines():
        line = line.strip()
        m = re.match(r"^(message|enum) (\w+) \{", line)
        if m:
            name = m.group(2)
            if m.group(1) == "message":
                messages[name] = dict(file=path.split("/")[-1], fields=[])
            else:
                enums[name] = []
            stack.append((m.group(1), name))
            continue
        if line.startswith("}"):
            if stack:
                stack.pop()
            continue
        if not stack:
            continue
        kind, name = stack[-1]
        if kind == "enum":
            m = re.match(r"^(\w+)\s*=\s*(-?\d+);", line)
            if m:
                enums[name].append((m.group(1), int(m.group(2))))
        else:
            m = re.match(r"^(optional|required|repeated)?\s*([\w.]+) (\w+)\s*=\s*(\d+);", line)
            if m:
                label, typ, fname, num = m.groups()
                messages[name]["fields"].append(dict(
                    number=int(num), name=fname, label=label or "",
                    kind="scalar" if typ in SCALARS else ("enum" if typ in enums else "message"),
                    type=typ,
                ))
for msg in messages.values():
    for f in msg["fields"]:
        if f["kind"] == "message" and f["type"] in enums:
            f["kind"] = "enum"
json.dump(dict(messages=messages, enums=enums), open(sys.argv[1], "w"), indent=1)
print("messages", len(messages), "enums", len(enums))
