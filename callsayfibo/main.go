package main

import (
	"fmt"
	"log"
	"os/exec"
	"time"
)

type Fibo struct {
	cache map[int]int
}

func New() *Fibo {
	initial := make(map[int]int)
	initial[0] = 0
	initial[1] = 1
	return &Fibo{
		cache: initial,
	}
}

func (f Fibo) GetAt(n int) int {
	if n < 2 {
		return n
	}
	if val, ok := f.cache[n]; ok {
		return val
	}
	b2 := f.GetAt(n - 2)
	b1 := f.GetAt(n - 1)
	f.cache[n] = b2 + b1
	return b2 + b1
}

func main() {
	fibo := New()
	current := 0
	for {
		cmd := exec.Command("say", fmt.Sprintf("%d", fibo.GetAt(current)))
		if err := cmd.Run(); err != nil {
			log.Fatal(err)
		}
		time.Sleep(time.Duration(time.Millisecond * 100))
		current += 1
	}
}
