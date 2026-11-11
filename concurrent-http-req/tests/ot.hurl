# 9. Operational transform: three clients edit "hello world" from version 0.
POST {{host}}/ot/docs
{"text": "hello world"}
HTTP 201
[Captures]
doc: header "Location"
[Asserts]
jsonpath "$.version" == 0

# A prepends.
POST {{host}}{{doc}}/ops
{"base_version": 0, "op": {"insert": {"pos": 0, "text": "Hey, "}}}
HTTP 201
[Asserts]
jsonpath "$.text" == "Hey, hello world"

# B deletes "world" at position 6 of the text it saw; shifted to 11 past A's insert.
POST {{host}}{{doc}}/ops
{"base_version": 0, "op": {"delete": {"pos": 6, "len": 5}}}
HTTP 201
[Asserts]
jsonpath "$.text" == "Hey, hello "
jsonpath "$.applied[0].delete.pos" == 11

# C appends "!" at the end of the text it saw.
POST {{host}}{{doc}}/ops
{"base_version": 0, "op": {"insert": {"pos": 11, "text": "!"}}}
HTTP 201
[Asserts]
jsonpath "$.text" == "Hey, hello !"
jsonpath "$.version" == 3

GET {{host}}{{doc}}
HTTP 200
[Asserts]
header "ETag" == "\"3\""
jsonpath "$.text" == "Hey, hello !"

# A concurrent insert inside a deleted range survives: the delete is split around it.
POST {{host}}/ot/docs
{"text": "abcdef"}
HTTP 201
[Captures]
doc2: header "Location"

POST {{host}}{{doc2}}/ops
{"base_version": 0, "op": {"insert": {"pos": 3, "text": "XYZ"}}}
HTTP 201

POST {{host}}{{doc2}}/ops
{"base_version": 0, "op": {"delete": {"pos": 1, "len": 4}}}
HTTP 201
[Asserts]
jsonpath "$.text" == "aXYZf"
jsonpath "$.applied" count == 2

# Overlapping concurrent deletes remove the union, not too much.
POST {{host}}/ot/docs
{"text": "abcdef"}
HTTP 201
[Captures]
doc3: header "Location"

POST {{host}}{{doc3}}/ops
{"base_version": 0, "op": {"delete": {"pos": 1, "len": 3}}}
HTTP 201

POST {{host}}{{doc3}}/ops
{"base_version": 0, "op": {"delete": {"pos": 2, "len": 3}}}
HTTP 201
[Asserts]
jsonpath "$.text" == "af"

# Invalid input.
POST {{host}}{{doc3}}/ops
{"base_version": 99, "op": {"insert": {"pos": 0, "text": "x"}}}
HTTP 422

POST {{host}}{{doc3}}/ops
{"base_version": 2, "op": {"insert": {"pos": 50, "text": "x"}}}
HTTP 422
