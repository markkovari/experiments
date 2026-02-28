Feature: Sub-Workflow Support

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: Register workflow with sub_workflow field
    When I POST to "/workflows" with body:
      """
      {"name":"parent-wf","steps":[
        {"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}
      ]}
      """
    Then the response status is 201
    And the response body contains "created"

  Scenario: Sub-workflow step appears in ready steps with kind sub_workflow
    Given I have registered a workflow named "parent2" with steps:
      """
      [{"name":"delegate","depends_on":[],"sub_workflow":"child-wf"}]
      """
    And I have started a run of "parent2"
    When I GET "/runs/{run_id}/ready"
    Then the response status is 200
    And the response body contains "sub_workflow"

  Scenario: Sub-workflow auto-completes when child run succeeds
    Given I have registered a workflow named "auto-parent" with steps:
      """
      [{"name":"child-step","depends_on":[],"sub_workflow":"child-wf"}]
      """
    And I have started a run of "auto-parent"
    When the child run for step "child-step" on run "{run_id}" succeeds
    Then the step "child-step" state is "succeeded"

  Scenario: Sub-workflow auto-fails when child run fails
    Given I have registered a workflow named "fail-parent" with steps:
      """
      [{"name":"child-step","depends_on":[],"sub_workflow":"child-wf"}]
      """
    And I have started a run of "fail-parent"
    When the child run for step "child-step" on run "{run_id}" fails
    Then the step "child-step" state is "failed"

  Scenario: Three-level nesting is accepted
    Given I have registered a workflow named "grandparent" with steps:
      """
      [{"name":"level1","depends_on":[],"sub_workflow":"parent-wf"}]
      """
    And I have registered a workflow named "parent-wf" with steps:
      """
      [{"name":"level2","depends_on":[],"sub_workflow":"child-wf"}]
      """
    And I have registered a workflow named "child-wf" with steps:
      """
      [{"name":"leaf","depends_on":[]}]
      """
    When I POST to "/runs" with body:
      """
      {"wf_name":"grandparent"}
      """
    Then the response status is 201
