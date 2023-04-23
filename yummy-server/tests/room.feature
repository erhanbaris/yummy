Feature: Authentication features


  Scenario: Need to login before creating new room
    Given user1 connected
    When user1 create new room for 4 player
    Then user1 request failed

  Scenario: Create basic room
    Given user1 connected
    When user1 register via email with "user1@gmail.com" "erhan"
    Then user1 email authenticated

    When user1 create new room for 4 player
    Then user1 receive room created message as room1


  Scenario: Create basic room
    Given user1 connected
    And user2 connected
    And user3 connected
    And user4 connected
    And user5 connected
    When user1 register via email with "user1@gmail.com" "erhan"
    And user2 register via email with "user2@gmail.com" "erhan"
    And user3 register via email with "user3@gmail.com" "erhan"
    And user4 register via email with "user4@gmail.com" "erhan"
    And user5 register via email with "user5@gmail.com" "erhan"
    Then user1 email authenticated
    And user2 email authenticated
    And user3 email authenticated
    And user4 email authenticated
    And user5 email authenticated

    When user1 create new room for 4 player
    Then user1 receive room created message as room1

    When user2 try to join room1 as User
    Then user2 joined to ""
    And user1 receive JoinToRoom message

    When user3 try to join room1 as User
    Then user3 joined to ""
    And user1 receive JoinToRoom message
    And user2 receive JoinToRoom message

    When user4 try to join room1 as User
    Then user4 joined to ""
    And user1 receive JoinToRoom message
    And user2 receive JoinToRoom message
    And user3 receive JoinToRoom message
    
    # Room reaches to maximum already. This user out of the room
    When user5 try to join room1 as User
    Then user5 request failed
