Feature: NATS Message System

  Scenario: Sending a message to NATS
    Given the NATS server is running
    When I publish a "cloudevents.hello" event with content "Hello, World!"
    Then the message should be received by the subscriber
