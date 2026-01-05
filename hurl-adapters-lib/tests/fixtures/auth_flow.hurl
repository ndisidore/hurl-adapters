POST https://api.example.com/auth/login
Content-Type: application/json
{
    "username": "testuser",
    "password": "{{PASSWORD}}"
}
HTTP 200
[Captures]
login_token: jsonpath "$.access_token"
login_user_id: jsonpath "$.user.id"
[Asserts]
jsonpath "$.access_token" exists

GET https://api.example.com/users/{{login_user_id}}
Authorization: Bearer {{login_token}}
HTTP 200
[Asserts]
jsonpath "$.id" == {{login_user_id}}
jsonpath "$.role" isString

