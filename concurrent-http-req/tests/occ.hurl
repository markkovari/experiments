# 1. Optimistic concurrency with ETag / If-Match.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# A and B both read the same ETag.
GET {{host}}/items/{{id}}
HTTP 200
[Captures]
etag: header "ETag"
[Asserts]
header "ETag" == "\"1\""

# A write without a precondition is refused.
PUT {{host}}/occ/items/{{id}}
{"name": "x", "qty": 0}
HTTP 428

# A writes first and wins.
PUT {{host}}/occ/items/{{id}}
If-Match: {{etag}}
{"name": "renamed-by-A", "qty": 10}
HTTP 200
[Asserts]
header "ETag" == "\"2\""

# B writes with the stale ETag: rejected instead of overwriting A.
PUT {{host}}/occ/items/{{id}}
If-Match: {{etag}}
{"name": "widget", "qty": 99}
HTTP 412

# B re-reads, reapplies its change on top of A's, and succeeds.
GET {{host}}/items/{{id}}
HTTP 200
[Captures]
etag2: header "ETag"

PUT {{host}}/occ/items/{{id}}
If-Match: {{etag2}}
{"name": "renamed-by-A", "qty": 99}
HTTP 200

GET {{host}}/items/{{id}}
HTTP 200
[Asserts]
jsonpath "$.name" == "renamed-by-A"
jsonpath "$.qty" == 99
jsonpath "$.version" == 3

# Unknown item is 404, not 412.
PUT {{host}}/occ/items/999999999
If-Match: "1"
{"name": "x", "qty": 0}
HTTP 404
