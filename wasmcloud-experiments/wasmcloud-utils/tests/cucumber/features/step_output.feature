Feature: Step Output Retrieval

  Background:
    Given the workflow API is running at "http://localhost:8080"
    And I have registered a workflow named "output-wf" with steps:
      """
      [{"name":"produce","depends_on":[]}]
      """

  Scenario: Retrieve step output after completion
    Given I have started a run of "output-wf"
    And I mark step "produce" as done with output "ImhlbGxvIg==" on run "{run_id}"
    When I GET "/runs/{run_id}/steps/produce/output"
    Then the response status is 200
    And the response body contains "hello"

  Scenario: Step output returns 404 for unknown run
    When I GET "/runs/no-such-run/steps/produce/output"
    Then the response status is 404

  Scenario: Step output returns 404 for unknown step
    Given I have started a run of "output-wf"
    When I GET "/runs/{run_id}/steps/ghost-step/output"
    Then the response status is 404

  Scenario: Step output shows pending state before step completes
    Given I have started a run of "output-wf"
    When I GET "/runs/{run_id}/steps/produce/output"
    Then the response status is 200
    And the response body contains "pending"
