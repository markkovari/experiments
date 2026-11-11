# 5. Server-side three-way merge against the version the client started from.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# Everyone edits from version 1. A changes the name.
PUT {{host}}/merge/items/{{id}}
{"base_version": 1, "name": "renamed-by-A", "qty": 10}
HTTP 200
[Asserts]
jsonpath "$.version" == 2

# B changed only qty: merged automatically, A's rename is kept.
PUT {{host}}/merge/items/{{id}}
{"base_version": 1, "name": "widget", "qty": 15}
HTTP 200
[Asserts]
jsonpath "$.name" == "renamed-by-A"
jsonpath "$.qty" == 15
jsonpath "$.version" == 3

# C also renamed, differently: a real conflict, returned for the client to resolve.
PUT {{host}}/merge/items/{{id}}
{"base_version": 1, "name": "renamed-by-C", "qty": 10}
HTTP 409
[Asserts]
jsonpath "$.conflicts" count == 1
jsonpath "$.conflicts[0].field" == "name"
jsonpath "$.conflicts[0].base" == "widget"
jsonpath "$.conflicts[0].mine" == "renamed-by-C"
jsonpath "$.conflicts[0].theirs" == "renamed-by-A"
jsonpath "$.current.version" == 3

# Same change on both sides is not a conflict.
PUT {{host}}/merge/items/{{id}}
{"base_version": 1, "name": "renamed-by-A", "qty": 10}
HTTP 200
[Asserts]
jsonpath "$.name" == "renamed-by-A"
jsonpath "$.qty" == 15

PUT {{host}}/merge/items/{{id}}
{"base_version": 99, "name": "x", "qty": 0}
HTTP 404
