#!/bin/bash
end=$((SECONDS+600))
while [ $SECONDS -lt $end ]; do
  # Pick a random rate between 2 and 40
  rate=$(( ( RANDOM % 39 )  + 2 ))
  echo "Current rate: $rate req/s (Time left: $(($end - $SECONDS))s)"
  
  for i in $(seq 1 $rate); do
    curl -s -X POST http://localhost:8080/order.v1.OrderService/CreateOrder \
      -H "Content-Type: application/json" \
      -H "Connect-Protocol-Version: 1" \
      -d "{\"customer_id\": \"fluctuating-user\", \"amount\": $rate.0}" > /dev/null &
  done
  sleep 1
done
