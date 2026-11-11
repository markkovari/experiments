# 6. Event sourcing with optimistic append on (stream_id, version).
POST {{host}}/es/streams/order-{{newUuid}}/events
{"expected_version": 0, "type": "OrderPlaced", "data": {"qty": 10}}
HTTP 201
[Captures]
events_url: url
stream_url: url replace "/events" ""
[Asserts]
jsonpath "$.version" == 1

# B also thought the stream was empty: conflict, told the current version.
POST {{events_url}}
{"expected_version": 0, "type": "OrderPlaced", "data": {"qty": 99}}
HTTP 409
[Asserts]
jsonpath "$.current_version" == 1

# B reloads, decides its event still makes sense, and appends at the right version.
POST {{events_url}}
{"expected_version": 1, "type": "QtyChanged", "data": {"qty": 12}}
HTTP 201
[Asserts]
jsonpath "$.version" == 2

# A version from the future is also rejected (no gaps).
POST {{events_url}}
{"expected_version": 5, "type": "QtyChanged", "data": {"qty": 1}}
HTTP 409

GET {{stream_url}}
HTTP 200
[Asserts]
header "ETag" == "\"2\""
jsonpath "$.version" == 2
jsonpath "$.events" count == 2
jsonpath "$.events[0].type" == "OrderPlaced"
jsonpath "$.events[1].data.qty" == 12

GET {{host}}/es/streams/does-not-exist
HTTP 404
