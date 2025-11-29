from collections import Counter
import json

with open("upstream/src/reference/v8.json") as f:
    data = json.load(f)

def get_type(value) -> list[str]:
    if isinstance(value, dict):
        t = value.get("type")
        if isinstance(t, str):
            return [t]
        types = []
        for v in value.values():
          for t in get_type(v):
            types.append(t)
        return types
    return []

types: list[str] = []
for value in data.values():
    for t in get_type(value):
        types.append(t)
for cnt, t in reversed(sorted((cnt,t) for (t,cnt) in Counter(types).items())):
    print(f"{cnt}: {t}")
