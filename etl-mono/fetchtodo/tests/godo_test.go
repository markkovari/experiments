package natscloudevents_nats

import (
	"testing"

	"github.com/cucumber/godog"
	"github.com/markkovari/experiments/etl-mono/steps"
)

func TestFeatures(t *testing.T) {
	opts := godog.Options{
		Format:   "pretty",
		Paths:    []string{"../features"},
		TestingT: t,
	}

	status := godog.TestSuite{
		Name:                "godog",
		ScenarioInitializer: steps.InitializeScenario,
		Options:             &opts,
	}.Run()

	if status != 0 {
		t.Fail()
	}
}
