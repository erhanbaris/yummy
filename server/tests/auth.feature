Feature: Authentication features

  Scenario: Email register
    Given user1 connected
    When user1 register email auth with "user1@gmail.com" "erhan"
    Then user1 receive Authenticated message

  Scenario: Email auth
    Given user1 connected
    When user1 email auth with "user1@gmail.com" "erhan"
    Then user1 request failed

    When user1 register email auth with "user1@gmail.com" "erhan"
    Then user1 receive Authenticated message

    When user1 logout
    Then user1 request succeeded
