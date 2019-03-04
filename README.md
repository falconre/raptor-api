# raptor-api

This provides a json-rpc interface to programs analyzed with raptor. You should absolutely build in release mode.

See the example `calls-to-symbol.py` script

```python
import raptor
import sys

store = raptor.Store('http://localhost:3030/')

document = store.document(sys.argv[1])

calls = document.calls_to_symbol(sys.argv[2])

for call in calls:
    print(call.function.name, call.instruction)
```

You can also use https://github.com/falconre/raptor-gui to view binaries in raptor-api.