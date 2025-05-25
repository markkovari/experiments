package main

import "fmt"

func Distance(word, another string) (int, error) {
	if len(word) != len(another) {
		return 0, fmt.Errorf("has to be the same length")
	}
	distance := 0
	for index, letter := range word {
		if []rune(another)[index] != letter {
			distance += 1
		}
	}
	return distance, nil
}

func Hamming(original string, words []string, distance int) []string {
	result := []string{}
	for _, word := range words {
		dist, err := Distance(word, original)
		if err == nil && dist <= distance {
			result = append(result, word)
		}
	}
	return result
}

func main() {
	println("HELLO HAMMING")
}
