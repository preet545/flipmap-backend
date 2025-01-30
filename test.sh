#!/usr/bin/env bash
# This routes cross-country to 'Computer Center' not sure why. My bit works though
curl -v -m 30 -X POST localhost:1337/route -H "Content-Type: application/json" -d "{\"lat\": 44.568760,\"lon\": -123.277961,\"query\":\"Milne Computer Center\"}"
