Feature: Authentication features


  Scenario: Email register
    Given user1 connected
    When user1 register via email with "user1@gmail.com" "erhan"
    Then user1 authenticated


  Scenario: Email auth
    Given user1 connected
    When user1 authenticate via email with "user1@gmail.com" "erhan"
    Then user1 request failed

    When user1 register via email with "" ""
    Then user1 request failed
    
    When user1 register via email with "user1@gmail.com" ""
    Then user1 request failed

    When user1 register via email with "user1@gmail.com" "erhan"
    Then user1 authenticated

    When user1 logout
    Then user1 request succeeded


  Scenario: Custom id auth
    Given user1 connected
    When user1 authenticate via custom id with "erhan"
    Then user1 request failed

    When user1 register via custom id with ""
    Then user1 request failed

    When user1 register via custom id with "user1@gmail.com"
    Then user1 authenticated

    When user1 logout
    Then user1 request succeeded


  Scenario: Device id auth
    Given user1 connected
    When user1 authenticate via device id with "my custom device id"
    Then user1 authenticated

    When user1 logout
    Then user1 request succeeded


  Scenario: Token refresh
    Given user1 connected
    When user1 try to refresh token
    Then user1 request failed

    When user1 authenticate via custom id with "token refresh"
    Then user1 authenticated

    When user1 try to refresh token
    Then user1 request succeeded


  Scenario: Token refresh failed
    Given user1 connected
    When user1 set token to "dummy token information"
    And user1 try to refresh token
    Then user1 request failed


  Scenario: Token restore
    Given user1 connected
    When user1 try to restore token
    Then user1 request failed

    When user1 set token to "dummy token information"
    And user1 try to restore token
    Then user1 request failed

    When user1 authenticate via custom id with "token refresh"
    Then user1 authenticated

    When user1 save token to memory
    And user1 try to restore token
    Then user1 request succeeded

  Scenario: Logout
    Given user1 connected
    When user1 logout
    Then user1 request failed
