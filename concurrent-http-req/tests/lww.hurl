# 0. Last-write-wins: B writes from a stale read and silently erases A's rename.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# A and B both read version 1 (implicit). A renames.
PUT {{host}}/lww/items/{{id}}
{"name": "renamed-by-A", "qty": 10}
HTTP 200

# B, still holding the old name, changes qty. No guard, so it succeeds.
PUT {{host}}/lww/items/{{id}}
{"name": "widget", "qty": 99}
HTTP 200

# A's update is lost.
GET {{host}}/items/{{id}}
HTTP 200
[Asserts]
jsonpath "$.name" == "widget"
jsonpath "$.qty" == 99
