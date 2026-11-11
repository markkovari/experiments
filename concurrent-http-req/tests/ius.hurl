# 2. If-Unmodified-Since: like If-Match, but with the Last-Modified timestamp.
# Weakness: 1 s resolution. Two writes within the same second both pass (see bench).
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# A and B both read the same Last-Modified.
GET {{host}}/items/{{id}}
HTTP 200
[Captures]
lm: header "Last-Modified"
[Asserts]
header "Last-Modified" exists

PUT {{host}}/ius/items/{{id}}
{"name": "x", "qty": 0}
HTTP 428

PUT {{host}}/ius/items/{{id}}
If-Unmodified-Since: not a date
{"name": "x", "qty": 0}
HTTP 400

# A writes in a later second than the read (the delay makes the timestamps differ).
PUT {{host}}/ius/items/{{id}}
If-Unmodified-Since: {{lm}}
[Options]
delay: 1100
{"name": "renamed-by-A", "qty": 10}
HTTP 200
[Asserts]
header "Last-Modified" exists

# B writes with the stale timestamp: rejected.
PUT {{host}}/ius/items/{{id}}
If-Unmodified-Since: {{lm}}
{"name": "widget", "qty": 99}
HTTP 412

# B re-reads and retries.
GET {{host}}/items/{{id}}
HTTP 200
[Captures]
lm2: header "Last-Modified"

PUT {{host}}/ius/items/{{id}}
If-Unmodified-Since: {{lm2}}
{"name": "renamed-by-A", "qty": 99}
HTTP 200
[Asserts]
jsonpath "$.version" == 3
