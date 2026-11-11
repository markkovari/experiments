# 3. Pessimistic lock with a lease.
POST {{host}}/items
{"name": "widget", "qty": 10}
HTTP 201
[Captures]
id: jsonpath "$.id"

# A takes the lock.
POST {{host}}/lock/items/{{id}}/lock
HTTP 201
[Captures]
tokenA: jsonpath "$.token"

# B cannot take it, and cannot write without it.
POST {{host}}/lock/items/{{id}}/lock
HTTP 423

PUT {{host}}/lock/items/{{id}}
{"name": "by-B", "qty": 1}
HTTP 423

PUT {{host}}/lock/items/{{id}}
Lock-Token: not-the-token
{"name": "by-B", "qty": 1}
HTTP 423

# A writes with its token.
PUT {{host}}/lock/items/{{id}}
Lock-Token: {{tokenA}}
{"name": "by-A", "qty": 11}
HTTP 200
[Asserts]
jsonpath "$.name" == "by-A"

# Only the holder can release.
DELETE {{host}}/lock/items/{{id}}/lock
Lock-Token: not-the-token
HTTP 423

DELETE {{host}}/lock/items/{{id}}/lock
Lock-Token: {{tokenA}}
HTTP 204

# B takes a short lease, then disappears without releasing.
POST {{host}}/lock/items/{{id}}/lock?ttl_secs=1
HTTP 201
[Captures]
tokenB: jsonpath "$.token"

POST {{host}}/lock/items/{{id}}/lock
HTTP 423

# After the lease expires, A can take the lock over...
POST {{host}}/lock/items/{{id}}/lock
[Options]
delay: 2100
HTTP 201

# ...and B's stale token no longer works.
PUT {{host}}/lock/items/{{id}}
Lock-Token: {{tokenB}}
{"name": "by-B", "qty": 1}
HTTP 423

POST {{host}}/lock/items/999999999/lock
HTTP 404
