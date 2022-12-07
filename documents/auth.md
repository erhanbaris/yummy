# Authentication

## Authenticate via email

### Request message

| Field name | Type    | Required | Description                                    | Default value |
|------------|---------|----------|------------------------------------------------|---------------|
| type       | string  | Y        | Value must be 'User'                           |               |
| auth_type  | string  | Y        | Value must be 'Email'                          |               |
| email      | string  | Y        | Authentication email address                   |               |
| password   | string  | Y        | Authentication password                        |               |
| create     | boolean | N        | If the user is not created yet, create new one | false         |

**Example request**

```json
{
    "type": "User",
    "auth_type": "Email",
    "email": "erhanbaris@gmail.com",
    "password": "erhan",
    "create": true
}
```

[Success response](#success-response)

[Fail response](#fail-response)


### Success Response

| Field name | Type    | Required | Description                 | Default value |
|------------|---------|----------|-----------------------------|---------------|
| status     | boolean | Y        | Value should be 'true'      |               |
| result     | string  | Y        | User's authentication token |               |

```json
{
    "status": true,
    "result": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJleHAiOjE2NzA1Mjg3MTEsInVzZXIiOnsiaWQiOiJiMDhkN2I3OS0xNDA1LTQxZGMtODJhMS02YTg4MjU3OWM3MmEiLCJzZXNzaW9uIjoiM2IxMzdjYWUtZmY1OC00NjY5LTg1YjctOWEyM2NiOGRiYzAxIiwibmFtZSI6bnVsbCwiZW1haWwiOiJlcmhhbmJhcmlzQGdtYWlsLmNvbSJ9fQ.6tLnsjWPRCz0cW00j2nzV-SUk6GwrlYgxe9V_p5mhxU"
}
```

### Fail Response

| Field name | Type    | Required | Description                 | Default value |
|------------|---------|----------|-----------------------------|---------------|
| status     | boolean | Y        | Value should be 'false'     |               |
| result     | string  | Y        | Error message               |


**Example request**:

```json
{
    "status": false,
    "result": "Email and/or password not valid"
}
```
