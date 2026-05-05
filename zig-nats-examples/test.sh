#!/bin/bash
TOKEN=$(curl -s -X POST http://192.168.139.2:8081/api/auth/login -H "Content-Type: application/json" -d '{"email": "test500@example.com", "password": "password123"}' | grep -o '"token":"[^"]*' | grep -o '[^"]*$')
echo "TOKEN=$TOKEN"
curl -s -v -X POST http://192.168.139.2:8081/api/orgs -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" -d '{"name": "My New Org"}'
