# 8. CRDTs: each replica sends its running totals; any order or duplicate converges.
POST {{host}}/crdt/counter-{{newUuid}}
{"replica": "a", "p": 3}
HTTP 200
[Captures]
key_url: url
[Asserts]
jsonpath "$.value" == 3

# Replica b: +2 and -1 so far.
POST {{key_url}}
{"replica": "b", "p": 2, "n": 1}
HTTP 200
[Asserts]
jsonpath "$.value" == 4
jsonpath "$.replicas" == 2

# Duplicate delivery of a's state: no double counting.
POST {{key_url}}
{"replica": "a", "p": 3}
HTTP 200
[Asserts]
jsonpath "$.value" == 4

# An older state of a arrives late: ignored.
POST {{key_url}}
{"replica": "a", "p": 1}
HTTP 200
[Asserts]
jsonpath "$.value" == 4

POST {{key_url}}
{"replica": "a", "p": 5}
HTTP 200
[Asserts]
jsonpath "$.value" == 6

# Last-writer-wins register: highest timestamp wins, replica id breaks ties.
POST {{key_url}}
{"replica": "a", "name": {"value": "first", "ts": 10}}
HTTP 200
[Asserts]
jsonpath "$.name" == "first"

POST {{key_url}}
{"replica": "b", "name": {"value": "older", "ts": 5}}
HTTP 200
[Asserts]
jsonpath "$.name" == "first"

POST {{key_url}}
{"replica": "b", "name": {"value": "tie-b", "ts": 10}}
HTTP 200
[Asserts]
jsonpath "$.name" == "tie-b"
jsonpath "$.value" == 6

GET {{key_url}}
HTTP 200
[Asserts]
jsonpath "$.value" == 6
jsonpath "$.name" == "tie-b"

POST {{key_url}}
{"replica": "a", "p": -1}
HTTP 422

GET {{host}}/crdt/does-not-exist
HTTP 404
