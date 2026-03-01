Feature: Workflow Definition Management

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: Register a minimal workflow
    When I POST to "/workflows" with body:
      """
      {"name":"simple-job","steps":[{"name":"run","depends_on":[]}]}
      """
    Then the response status is 201
    And the response body contains "created"

  Scenario: List workflows returns registered definition
    Given I have registered a workflow named "list-test"
    When I GET "/workflows"
    Then the response status is 200
    And the response body contains "list-test"

  Scenario: Get a specific workflow by name
    Given I have registered a workflow named "get-test"
    When I GET "/workflows/get-test"
    Then the response status is 200
    And the response body contains "get-test"

  Scenario: Delete a workflow
    Given I have registered a workflow named "delete-test"
    When I DELETE "/workflows/delete-test"
    Then the response status is 204

  Scenario: Get returns 404 for unknown workflow
    When I GET "/workflows/nonexistent-workflow"
    Then the response status is 404

  Scenario: Reject workflow with empty name
    When I POST to "/workflows" with body:
      """
      {"name":"","steps":[{"name":"run","depends_on":[]}]}
      """
    Then the response status is 400
    And the response body contains "name must not be empty"

  Scenario: Reject workflow with invalid name characters
    When I POST to "/workflows" with body:
      """
      {"name":"bad name!","steps":[{"name":"run","depends_on":[]}]}
      """
    Then the response status is 400
    And the response body contains "invalid characters"

  Scenario: Reject workflow with no steps
    When I POST to "/workflows" with body:
      """
      {"name":"empty-steps","steps":[]}
      """
    Then the response status is 400
    And the response body contains "at least one step"

  Scenario: Reject workflow with duplicate step names
    When I POST to "/workflows" with body:
      """
      {"name":"dup-steps","steps":[
        {"name":"a","depends_on":[]},
        {"name":"a","depends_on":[]}
      ]}
      """
    Then the response status is 400
    And the response body contains "duplicate step name"

  Scenario: Reject workflow with unknown dependency
    When I POST to "/workflows" with body:
      """
      {"name":"bad-dep","steps":[{"name":"a","depends_on":["nonexistent"]}]}
      """
    Then the response status is 400
    And the response body contains "unknown step"

  Scenario: Reject workflow with max_attempts zero
    When I POST to "/workflows" with body:
      """
      {"name":"zero-attempts","steps":[{"name":"s","depends_on":[],"max_attempts":0}]}
      """
    Then the response status is 400
    And the response body contains "max_attempts must be >= 1"

  Scenario: Reject workflow with cyclic dependency
    When I POST to "/workflows" with body:
      """
      {"name":"wf","steps":[
        {"name":"a","depends_on":["b"]},
        {"name":"b","depends_on":["a"]}
      ]}
      """
    Then the response status is 400
    And the response body contains "cycle"

  Scenario: Reject workflow with invalid sub_workflow name
    When I POST to "/workflows" with body:
      """
      {"name":"bad-sub","steps":[{"name":"s","depends_on":[],"sub_workflow":"bad sub!"}]}
      """
    Then the response status is 400
    And the response body contains "invalid characters"

  Scenario: Reject workflow with condition referencing unknown step
    When I POST to "/workflows" with body:
      """
      {"name":"bad-cond","steps":[
        {"name":"s","depends_on":[],"condition":{"on_step":"ghost","equals":true}}
      ]}
      """
    Then the response status is 400
    And the response body contains "unknown step"
