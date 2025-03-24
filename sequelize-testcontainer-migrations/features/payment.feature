Feature: A user pays for another user

  Scenario: A user sends a payment to another user successfully
    Given a user with email: a@gmail.com and 100 as amount
    And a user with email: b@gmail.com and 50 as amount
    When the user with email: a@gmail.com pays to the user with: b@gmail.com with 20
    And the payment is successfully registered
    Then the user with email: a@gmail.com has 20 less on their account
    And the user with email: b@gmail.com has 20 more on their account
