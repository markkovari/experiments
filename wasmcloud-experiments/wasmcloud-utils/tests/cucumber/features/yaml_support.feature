Feature: YAML Content-Type Support

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: Register workflow via YAML body
    When I POST to "/workflows" with content-type "application/yaml" and body:
      """
      name: yaml-wf
      steps:
        - name: run
          depends_on: []
      """
    Then the response status is 201
    And the response body contains "created"

  Scenario: Register workflow via text/yaml content-type
    When I POST to "/workflows" with content-type "text/yaml" and body:
      """
      name: text-yaml-wf
      steps:
        - name: step1
          depends_on: []
      """
    Then the response status is 201
    And the response body contains "created"

  Scenario: Subscribe to event via YAML body
    When I POST to "/events/yaml.event/subscribe" with content-type "application/yaml" and body:
      """
      fn_name: yaml-handler
      """
    Then the response status is 204

  Scenario: Invalid YAML returns 400
    When I POST to "/workflows" with content-type "application/yaml" and body:
      """
      name: [this is: invalid: yaml
      steps: !!!
      """
    Then the response status is 400
    And the response body contains "invalid YAML"

  Scenario: YAML with cyclic dependency returns 400
    When I POST to "/workflows" with content-type "application/yaml" and body:
      """
      name: cyclic-yaml
      steps:
        - name: a
          depends_on: [b]
        - name: b
          depends_on: [a]
      """
    Then the response status is 400
    And the response body contains "cycle"
