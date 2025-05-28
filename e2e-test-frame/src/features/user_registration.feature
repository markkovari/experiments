Feature: User Registration

  Scenario: New user successfully registers on the platform
    Given I am on the registration page
    When I fill in the registration form with my details
    And I submit the registration form
    Then I should see a success message
    And I should be redirected to my dashboard

  Scenario: Attempt to register with an already existing email
    Given I am on the registration page
    And a user with email "existing@testmail.com" already exists
    When I fill in the registration form with email "existing@testmail.com" and other details
    And I submit the registration form
    Then I should see an error message indicating the email is already taken
