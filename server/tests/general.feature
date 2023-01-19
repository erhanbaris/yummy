Feature: General feature test

  Scenario: Empty message
    Given user1 connected

    When user1 send "" as a json message
    Then user1 request failed

    When user1 send "{}" as a json message
    Then user1 request failed

  Scenario: Wrong type
    Given user1 connected

    When user1 send '{"type": ""}' as a json message
    Then user1 request failed
    
    When user1 send '{"type": "ASDF"}' as a json message
    Then user1 request failed
