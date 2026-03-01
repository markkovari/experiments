Feature: Step and Run Timeouts

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: Reject workflow with run-level timeout_ms zero
    When I POST to "/workflows" with body:
      """
      {"name":"bad-timeout","steps":[{"name":"s","depends_on":[]}],"timeout_ms":0}
      """
    Then the response status is 400
    And the response body contains "timeout_ms must be > 0"

  Scenario: Reject workflow with step-level timeout_ms zero
    When I POST to "/workflows" with body:
      """
      {"name":"bad-step-timeout","steps":[{"name":"s","depends_on":[],"timeout_ms":0}]}
      """
    Then the response status is 400
    And the response body contains "timeout_ms must be > 0"

  Scenario: Step with timeout_ms field is accepted
    When I POST to "/workflows" with body:
      """
      {"name":"timeout-wf","steps":[{"name":"s","depends_on":[],"timeout_ms":5000}]}
      """
    Then the response status is 201

  Scenario: Run-level timeout field is accepted
    When I POST to "/workflows" with body:
      """
      {"name":"run-timeout-wf","timeout_ms":10000,"steps":[{"name":"s","depends_on":[]}]}
      """
    Then the response status is 201
