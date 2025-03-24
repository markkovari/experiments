Feature: A user pays for another user

  Scenario: A user sends a payment to another user successfully
    Given there are no users in the system
    And a user with email: <firstUser> and <firstUserAmount> as amount
    And a user with email: <secondUser> and <secondUserAmount> as amount
    When the user with email: <firstUser> pays to the user with: <secondUser> with <paymentAmount>
    And the payment is successfully registered from <firstUser> to <secondUser>
    Then the user with email: <firstUser> has <paymentAmount> less on their account
    And the user with email: <secondUser> has <paymentAmount> more on their account

    Examples:
      | firstUser     | firstUserAmount | secondUser    | secondUserAmount | paymentAmount |
      | "a@gmail.com" |            1000 | "b@gmail.com" |              120 |            15 |
      | "b@gmail.com" |             100 | "c@gmail.com" |              120 |            20 |
      | "c@gmail.com" |      1000000000 | "d@gmail.com" |        120000000 |      20000000 |
      | "c@gmail.com" |           99999 | "e@gmail.com" |            44444 |           111 |
