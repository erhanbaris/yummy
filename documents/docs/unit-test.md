
# Unit test executions 

To execute all unit test, need to execute following commands. The second command requires Redis instance.

!!! command "Execute"
    ```bash
    cargo test --all
    cargo test --all  --features stateless
    ```