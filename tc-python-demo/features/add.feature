Feature: adding cusotmers to the application works

  @fixture.postgres
  Scenario Outline: Adding customers
    Given an empty customers table
    When I add <amount> users to the database
    Then it should have <customers_amount> cusotmer in the database

    Examples: User amounts
      | amount | customers_amount |
      | 1      | 1                |
      | 10     | 10               |
      | 100    | 100              |
      | 1000   | 1000             |
