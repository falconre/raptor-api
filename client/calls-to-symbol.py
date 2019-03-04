import raptor
import sys

store = raptor.Store('http://localhost:3030/')

document = store.document(sys.argv[1])

calls = document.calls_to_symbol(sys.argv[2])

for call in calls:
    print(call.function.name, call.instruction)