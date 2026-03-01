Feature: Pagination

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: List workflows returns pagination metadata
    Given I have registered a workflow named "pag-wf-1"
    And I have registered a workflow named "pag-wf-2"
    When I GET "/workflows?page=1&limit=1"
    Then the response status is 200
    And the response body contains "items"
    And the response body contains "total"
    And the response body contains "page"
    And the response body contains "limit"

  Scenario: List steps for a run
    Given I have registered a workflow named "steps-list-wf" with steps:
      """
      [{"name":"a","depends_on":[]},{"name":"b","depends_on":[]}]
      """
    And I have started a run of "steps-list-wf"
    When I GET "/runs/{run_id}/steps"
    Then the response status is 200
    And the response body contains "items"
