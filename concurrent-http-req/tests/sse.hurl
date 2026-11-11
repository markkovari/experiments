# +. Change notifications over Server-Sent Events.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

PATCH {{host}}/patch/items/{{id}}
{"qty_inc": 1}
HTTP 200

# A subscriber first receives the current state. limit=1 closes the stream after it.
# (Hurl runs requests in order, so it can't observe a push caused by a concurrent write.)
GET {{host}}/items/{{id}}/events?limit=1
HTTP 200
[Asserts]
header "Content-Type" contains "text/event-stream"
body contains "event: item"
body contains "id: 2"
body contains "\"qty\":11"

GET {{host}}/items/999999999/events
HTTP 404
