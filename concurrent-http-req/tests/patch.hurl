# 4. Partial + relative updates: concurrent edits to different fields and
# concurrent increments all survive, no read-before-write needed.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# A renames, B adds 5, C removes 3. None of them sent the full object.
PATCH {{host}}/patch/items/{{id}}
{"name": "renamed-by-A"}
HTTP 200

PATCH {{host}}/patch/items/{{id}}
{"qty_inc": 5}
HTTP 200

PATCH {{host}}/patch/items/{{id}}
{"qty_inc": -3}
HTTP 200

GET {{host}}/items/{{id}}
HTTP 200
[Asserts]
jsonpath "$.name" == "renamed-by-A"
jsonpath "$.qty" == 12
jsonpath "$.version" == 4

# An absolute set can still be guarded, like a JSON Patch `test` op.
PATCH {{host}}/patch/items/{{id}}
{"qty": 0, "expected_version": 1}
HTTP 409

PATCH {{host}}/patch/items/{{id}}
{"qty": 0, "expected_version": 4}
HTTP 200
[Asserts]
jsonpath "$.qty" == 0

PATCH {{host}}/patch/items/999999999
{"qty_inc": 1}
HTTP 404
