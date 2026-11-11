# 7. Command queue: writes are accepted as commands and applied one at a time.
POST {{host}}/items
{"name": "counter", "qty": 0}
HTTP 201
[Captures]
id: jsonpath "$.id"

# Three clients fire increments back to back; each gets 202 + a status URL.
POST {{host}}/queue/items/{{id}}/commands
{"qty_inc": 1}
HTTP 202
[Captures]
cmd1: header "Location"
[Asserts]
jsonpath "$.status" == "pending"

POST {{host}}/queue/items/{{id}}/commands
{"qty_inc": 1}
HTTP 202
[Captures]
cmd2: header "Location"

POST {{host}}/queue/items/{{id}}/commands
{"qty_inc": 1}
HTTP 202
[Captures]
cmd3: header "Location"

# A command whose precondition is stale gets rejected by the worker.
POST {{host}}/queue/items/{{id}}/commands
{"qty": 100, "expected_version": 1}
HTTP 202
[Captures]
cmd4: header "Location"

# Poll until processed (commands run in order, so cmd4 last).
GET {{host}}{{cmd4}}
[Options]
retry: 20
retry-interval: 50
HTTP 200
[Asserts]
jsonpath "$.status" != "pending"

GET {{host}}{{cmd1}}
HTTP 200
[Asserts]
jsonpath "$.status" == "done"

GET {{host}}{{cmd2}}
HTTP 200
[Asserts]
jsonpath "$.status" == "done"

GET {{host}}{{cmd3}}
HTTP 200
[Asserts]
jsonpath "$.status" == "done"

GET {{host}}{{cmd4}}
HTTP 200
[Asserts]
jsonpath "$.status" == "failed"
jsonpath "$.error" exists

GET {{host}}/items/{{id}}
HTTP 200
[Asserts]
jsonpath "$.qty" == 3
jsonpath "$.version" == 4

GET {{host}}/commands/999999999
HTTP 404
