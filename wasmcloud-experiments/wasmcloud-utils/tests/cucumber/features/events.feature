Feature: Event Subscription and Emission

  Background:
    Given the workflow API is running at "http://localhost:8080"

  Scenario: Subscribe to an event
    When I POST to "/events/order.placed/subscribe" with body:
      """
      {"fn_name":"handle-order"}
      """
    Then the response status is 200
    And the response body contains "subscribed"

  Scenario: Emit an event to subscribers
    Given I have subscribed "handle-invoice" to event "invoice.created"
    When I POST to "/events/invoice.created/emit" with body:
      """
      {"payload":"eyJhbW91bnQiOjEwMH0="}
      """
    Then the response status is 200
    And the response body contains "emitted"

  Scenario: Unsubscribe from an event
    Given I have subscribed "handle-order" to event "order.placed"
    When I POST to "/events/order.placed/unsubscribe" with body:
      """
      {"fn_name":"handle-order"}
      """
    Then the response status is 200
    And the response body contains "unsubscribed"

  Scenario: List event subscribers
    Given I have subscribed "handler-a" to event "my.event"
    And I have subscribed "handler-b" to event "my.event"
    When I GET "/events/my.event/subscribers"
    Then the response status is 200
    And the response body contains "handler-a"
    And the response body contains "handler-b"

  Scenario: List subscribers for event with no subscribers returns empty list
    When I GET "/events/no-subscribers.event/subscribers"
    Then the response status is 200
    And the response body contains "[]"

  Scenario: Emit to event with no subscribers returns ok
    When I POST to "/events/nobody-listening/emit" with body:
      """
      {"payload":""}
      """
    Then the response status is 200
