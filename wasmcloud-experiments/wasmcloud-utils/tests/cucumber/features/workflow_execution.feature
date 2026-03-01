Feature: Workflow Execution Lifecycle

  Background:
    Given the workflow API is running at "http://localhost:8080"
    And I have registered a workflow named "exec-wf" with steps:
      """
      [{"name":"step-a","depends_on":[]},{"name":"step-b","depends_on":["step-a"]}]
      """

  Scenario: Start a workflow run
    When I POST to "/runs" with body:
      """
      {"wf_name":"exec-wf"}
      """
    Then the response status is 201
    And the response body contains "run_id"
    And I save the run_id

  Scenario: Get run status
    Given I have started a run of "exec-wf"
    When I GET "/runs/{run_id}"
    Then the response status is 200
    And the response body contains "running"

  Scenario: Cancel a running workflow
    Given I have started a run of "exec-wf"
    When I POST to "/runs/{run_id}/cancel" with body:
      """
      {}
      """
    Then the response status is 204

  Scenario: Idempotency key prevents duplicate runs
    When I POST to "/runs" with body:
      """
      {"wf_name":"exec-wf","idem_key":"my-unique-key-123"}
      """
    Then the response status is 201
    And I save the run_id
    When I POST to "/runs" with body:
      """
      {"wf_name":"exec-wf","idem_key":"my-unique-key-123"}
      """
    Then the response status is 200
    And the run_id matches the previously saved run_id

  Scenario: Ready steps for a new run
    Given I have started a run of "exec-wf"
    When I GET "/runs/{run_id}/ready"
    Then the response status is 200
    And the response body contains "step-a"

  Scenario: Mark a step as done
    Given I have started a run of "exec-wf"
    When I POST to "/runs/{run_id}/steps/step-a/done" with body:
      """
      {"output":"c3VjY2Vzcw=="}
      """
    Then the response status is 204

  Scenario: Mark a step as failed
    Given I have started a run of "exec-wf"
    When I POST to "/runs/{run_id}/steps/step-a/failed" with body:
      """
      {"error":"something went wrong"}
      """
    Then the response status is 204

  Scenario: Retry after failure resets attempt count
    Given I have started a run of "exec-wf"
    And step "step-a" has failed on run "{run_id}"
    When I POST to "/runs/{run_id}/steps/step-a/retry" with body:
      """
      {}
      """
    Then the response status is 204

  Scenario: Get unknown run returns 404
    When I GET "/runs/nonexistent-run-id"
    Then the response status is 404

  Scenario: Start run for unknown workflow returns 404
    When I POST to "/runs" with body:
      """
      {"wf_name":"workflow-that-does-not-exist"}
      """
    Then the response status is 404
    And the response body contains "not found"
