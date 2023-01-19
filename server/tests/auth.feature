Feature: Authentication features

  Scenario: Email register
    Given user1 connected
    When user1 register via email with "user1@gmail.com" "erhan"
    Then user1 receive Authenticated message

  Scenario: Email auth
    Given user1 connected
    When user1 authenticate via email with "user1@gmail.com" "erhan"
    Then user1 request failed

    When user1 register via email with "" ""
    Then user1 request failed
    
    When user1 register via email with "user1@gmail.com" ""
    Then user1 request failed

    When user1 register via email with "user1@gmail.com" "erhan"
    Then user1 receive Authenticated message

    When user1 logout
    Then user1 request succeeded


  Scenario: Custom id auth
    Given user1 connected
    When user1 authenticate via custom id with "erhan"
    Then user1 request failed

    When user1 register via custom id with ""
    Then user1 request failed

    When user1 register via custom id with "user1@gmail.com"
    Then user1 receive Authenticated message

    When user1 logout
    Then user1 request succeeded


  Scenario: Logout
    Given user1 connected

    When user1 logout
    Then user1 request failed
