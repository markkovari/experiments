package main

import (
	"fmt"
	"html"
	"log"
	"markkovari/callsayfibo/internal/say"
	"markkovari/callsayfibo/pkg/fibonacci"
	"net/http"
	"strconv"
	"strings"
)

func main() {

	fibo := fibonacci.New()

	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		trimmed := strings.TrimPrefix(r.URL.Path, "/")
		num, err := strconv.ParseInt(html.EscapeString(trimmed), 10, 64)
		if err != nil {
			say.Say(fmt.Sprintf("%s is not a number", trimmed))
			fmt.Fprintf(w, "Not a number %s", html.EscapeString(trimmed))
		}
		result := fibo.Get(num)
		asString := fmt.Sprintf("The result is %d", result)
		say.Say(asString)
		fmt.Fprint(w, asString)
	})

	fmt.Println("Server listening on port 8080")
	log.Fatal(http.ListenAndServe(":8080", nil))

}
