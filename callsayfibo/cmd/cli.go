package main

import (
	"fmt"
	"log"
	"markkovari/callsayfibo/pkg/fibonacci"
	"os/exec"
	"time"
)

func main() {
	fibo := fibonacci.New()
	current := int64(0)
	for {
		cmd := exec.Command("say", fmt.Sprintf("%d", fibo.Get(current)))
		if err := cmd.Run(); err != nil {
			log.Fatal(err)
		}
		time.Sleep(time.Duration(time.Millisecond * 100))
		current += 1
	}
}
