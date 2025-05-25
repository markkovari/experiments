package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
)

type Reviews struct {
	MatReview    float64 `json:"matReview"`
	PresSkills   float64 `json:"presSkills"`
	Helpfulness  float64 `json:"helpfulness"`
	Explanations float64 `json:"explanations"`
}

func fromText(lines []string) ([]Reviews, error) {
	reviews := []Reviews{}
	for _, line := range lines {
		rev, err := fromLine(line)
		if err == nil {
			reviews = append(reviews, *rev)
		}
	}
	return reviews, nil
}

func (rev Reviews) add(other Reviews) Reviews {
	return Reviews{
		MatReview:    rev.MatReview + other.MatReview,
		PresSkills:   rev.PresSkills + other.PresSkills,
		Helpfulness:  rev.Helpfulness + other.Helpfulness,
		Explanations: rev.Explanations + other.Explanations,
	}
}

func (rev Reviews) devideBy(by int) Reviews {
	return Reviews{
		MatReview:    rev.MatReview / float64(by),
		PresSkills:   rev.PresSkills / float64(by),
		Helpfulness:  rev.Helpfulness / float64(by),
		Explanations: rev.Explanations / float64(by),
	}
}

func AverageReviews(input []Reviews) Reviews {
	devider := len(input)
	sum := Reviews{MatReview: 0, PresSkills: 0, Helpfulness: 0, Explanations: 0}
	for _, input := range input {
		sum = sum.add(input)
	}
	return sum.devideBy(devider)
}

func fromLine(line string) (*Reviews, error) {
	splitted := strings.Split(line, " ")
	if len(splitted) != 4 {
		return nil, fmt.Errorf("Needs four elements")
	}
	mr, err := strconv.ParseFloat(splitted[0], 10)
	if err != nil {
		return nil, err
	}
	ps, err := strconv.ParseFloat(splitted[1], 10)
	if err != nil {
		return nil, err
	}
	h, err := strconv.ParseFloat(splitted[2], 10)
	if err != nil {
		return nil, err
	}
	e, err := strconv.ParseFloat(splitted[3], 10)
	if err != nil {
		return nil, err
	}

	return &Reviews{
		MatReview:    mr,
		PresSkills:   ps,
		Helpfulness:  h,
		Explanations: e,
	}, nil

}

func main() {
	content, err := os.ReadFile("./cmd/review/reviews.txt")
	if err != nil {
		empty, _ := json.Marshal(Reviews{
			MatReview:    0,
			PresSkills:   0,
			Helpfulness:  0,
			Explanations: 0,
		})
		os.Stderr.Write(empty)
		os.Exit(1)
	}
	lines := strings.Split(string(content), "\n")

	if len(lines) == 0 {
		empty, _ := json.Marshal(Reviews{
			MatReview:    0,
			PresSkills:   0,
			Helpfulness:  0,
			Explanations: 0,
		})
		os.Stderr.Write(empty)
		os.Exit(1)
	}
	reviews, err := fromText(lines)
	if err != nil {
		empty, _ := json.Marshal(Reviews{
			MatReview:    0,
			PresSkills:   0,
			Helpfulness:  0,
			Explanations: 0,
		})
		os.Stderr.Write(empty)
		os.Exit(1)
	} else {
		aggReview := AverageReviews(reviews)
		aggregatedReviewContent, err := json.Marshal(aggReview)
		if err != nil {
			empty, _ := json.Marshal(Reviews{
				MatReview:    0,
				PresSkills:   0,
				Helpfulness:  0,
				Explanations: 0,
			})
			os.Stderr.Write(empty)
			os.Exit(1)
		} else {
			os.Stdout.Write(aggregatedReviewContent)
		}
	}
}
