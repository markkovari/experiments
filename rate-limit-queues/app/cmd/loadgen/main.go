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
	"sync"
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
	maxConcurrent := 100
	sem := make(chan struct{}, maxConcurrent)

	var wg sync.WaitGroup

	for i := int64(0); i < untilAsNum; i++ {
		wg.Add(1)
		sem <- struct{}{} // acquire slot

		go func(i int64) {
			defer wg.Done()
			defer func() { <-sem }() // release slot
			var userId int
			if i%3 == 0 {
				userId = 0
			} else if i%5 == 0 {
				userId = 1
			} else {
				userId = int(i)
			}

			payload := map[string]any{
				"name":          "Mark",
				"email":         fmt.Sprintf("mark_%f_%d@example.com", seed, i),
				"passthroughId": userId,
			}

			jsonData, err := json.Marshal(payload)
			if err != nil {
				slog.Error("marshal failed", slog.String("err", err.Error()))
				return
			}

			req, err := http.NewRequest("POST", url, bytes.NewBuffer(jsonData))
			if err != nil {
				slog.Error("request creation failed", slog.String("err", err.Error()))
				return
			}
			req.Header.Set("Content-Type", "application/json")

			client := &http.Client{}
			resp, err := client.Do(req)
			if err != nil {
				slog.Error("http request failed", slog.String("err", err.Error()))
				return
			}
			defer resp.Body.Close()

			fmt.Printf("Request %d Status: %s\n", i, resp.Status)
		}(i)
	}

	wg.Wait()

}
