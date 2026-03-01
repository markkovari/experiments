Feature: Run History

  Background:
    Given the workflow API is running at "http://localhost:8080"
    And I have registered a workflow named "history-wf" with steps:
      """
      [{"name":"s","depends_on":[]}]
      """

  Scenario: List runs returns empty for fresh workflow
    When I GET "/workflows/history-wf/runs"
    Then the response status is 200
    And the response body contains "\"total\":0"

  Scenario: List runs for a workflow
    Given I have started a run of "history-wf"
    When I GET "/workflows/history-wf/runs"
    Then the response status is 200
    And the response body contains "items"

  Scenario: Filter runs by state
    Given I have started a run of "history-wf"
    When I GET "/workflows/history-wf/runs?state=running"
    Then the response status is 200
    And the response body contains "running"

  Scenario: Unknown workflow returns empty list
    When I GET "/workflows/no-such-workflow/runs"
    Then the response status is 200
    And the response body contains "\"total\":0"
