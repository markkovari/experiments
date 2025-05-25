package main

import (
	"reflect"
	"testing"
)

func TestHammingSimple(t *testing.T) {
	oneDist, err := Distance("apple", "appla")
	if err != nil {
		t.Fatal("apple an appla should have a hamming distance")
	}
	if oneDist != 1 {
		t.Fatalf("apple and appla should have one distance, instead got=%d", oneDist)
	}
}

func TestHammingError(t *testing.T) {
	_, err := Distance("appleasdasd", "appla")
	if err == nil {
		t.Fatal("appleasdasd an appla should have no hamming distance")
	}
}

func TestHammingOnelement(t *testing.T) {
	_, err := Distance("appleasdasd", "appla")
	if err == nil {
		t.Fatal("appleasdasd an appla should have no hamming distance")
	}
}

func TestHammingCorrect(t *testing.T) {
	words := Hamming("apple", []string{"apple", "apply", "tuple", "alter"}, 3)

	expected := []string{"apple", "apply", "tuple"}
	if !reflect.DeepEqual(words, expected) {
		t.Fatal("appleasdasd an appla should have no hamming distance")
	}
}
