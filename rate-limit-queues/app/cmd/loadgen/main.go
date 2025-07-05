package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log/slog"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"time"
)

const url = "http://localhost:8080/users"

func main() {
	rand.New(rand.NewSource(time.Now().UnixNano()))
	seed := rand.Float32()

	println(os.Args)
	until := os.Args[1]

	untilAsNum, err := strconv.ParseInt(until, 10, 64)
	if err != nil {
		slog.Error(" last argument have to be a number")
	}
	for i := range untilAsNum {
		// Struct to encode to JSON
		payload := map[string]string{
			"name":  "Mark",
			"email": fmt.Sprintf("mark_%f_%d@example.com", seed, i),
		}

		// Encode payload to JSON
		jsonData, err := json.Marshal(payload)
		if err != nil {
			panic(err)
		}

		// Create the request
		req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
		if err != nil {
			panic(err)
		}

		// Set headers
		req.Header.Set("Content-Type", "application/json")
		req.Header.Set("X-Custom-Header", "foobar") // optional

		// Send the request
		client := &http.Client{}
		resp, err := client.Do(req)
		if err != nil {
			panic(err)
		}
		defer resp.Body.Close()

		// Print response
		fmt.Println("Status:", resp.Status)
	}
}
