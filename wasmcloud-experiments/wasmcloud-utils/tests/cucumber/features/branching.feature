Feature: Conditional Branching

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: If-else true branch executes when condition met
    Given I have registered a workflow named "branch-true" with steps:
      """
      [
        {"name":"check","depends_on":[]},
        {"name":"on-true","depends_on":["check"],"condition":{"on_step":"check","equals":"yes"}},
        {"name":"on-false","depends_on":["check"],"condition":{"on_step":"check","equals":"no"}}
      ]
      """
    And I have started a run of "branch-true"
    And I mark step "check" as done with output "InllcyI=" on run "{run_id}"
    When I GET "/runs/{run_id}/ready"
    Then the response status is 200
    And the response body contains "on-true"
    And the response body does not contain "on-false"

  Scenario: If-else false branch executes when condition not met
    Given I have registered a workflow named "branch-false" with steps:
      """
      [
        {"name":"check","depends_on":[]},
        {"name":"on-true","depends_on":["check"],"condition":{"on_step":"check","equals":"yes"}},
        {"name":"on-false","depends_on":["check"],"condition":{"on_step":"check","equals":"no"}}
      ]
      """
    And I have started a run of "branch-false"
    And I mark step "check" as done with output "Im5vIg==" on run "{run_id}"
    When I GET "/runs/{run_id}/ready"
    Then the response status is 200
    And the response body contains "on-false"
    And the response body does not contain "on-true"

  Scenario: Optional steps do not fail the run when skipped
    Given I have registered a workflow named "optional-wf" with steps:
      """
      [
        {"name":"main","depends_on":[]},
        {"name":"optional-notify","depends_on":["main"],"optional":true,"condition":{"on_step":"main","equals":"notify"}}
      ]
      """
    And I have started a run of "optional-wf"
    And I mark step "main" as done with output "InNraXAi" on run "{run_id}"
    When I GET "/runs/{run_id}"
    Then the response status is 200
    And the response body contains "succeeded"

  Scenario: Transitive skip propagates through dependent steps
    Given I have registered a workflow named "transitive-skip" with steps:
      """
      [
        {"name":"gate","depends_on":[]},
        {"name":"mid","depends_on":["gate"],"condition":{"on_step":"gate","equals":"go"}},
        {"name":"leaf","depends_on":["mid"]}
      ]
      """
    And I have started a run of "transitive-skip"
    And I mark step "gate" as done with output "InN0b3Ai" on run "{run_id}"
    When I GET "/runs/{run_id}"
    Then the response status is 200
    And the response body contains "succeeded"
