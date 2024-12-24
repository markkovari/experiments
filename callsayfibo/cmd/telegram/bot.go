package main

import (
	"fmt"
	"log"
	"markkovari/callsayfibo/internal/say"
	"markkovari/callsayfibo/pkg/fibonacci"
	"os"
	"strconv"
	"time"

	tele "gopkg.in/telebot.v4"
)

func main() {
	fibo := fibonacci.New()
	pref := tele.Settings{
		Token:  os.Getenv("TELEGRAM_TOKEN"),
		Poller: &tele.LongPoller{Timeout: 10 * time.Second},
	}

	b, err := tele.NewBot(pref)
	if err != nil {
		log.Fatal(err)
		return
	}

	b.Handle("/fibo", func(c tele.Context) error {
		asString := c.Args()[0]
		parsed, err := strconv.ParseInt(asString, 10, 64)
		if err != nil {
			return c.Send(fmt.Sprintf("%s is not a number try with a number", asString))
		}
		result := fmt.Sprintf("The result is: %d", fibo.Get(parsed))
		say.Say(result)
		return c.Send(result)
	})

	b.Start()
}
