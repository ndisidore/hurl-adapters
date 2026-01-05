POST https://api.example.com/auth/login
Content-Type: application/json
{
    "username": "testuser",
    "password": {{PASSWORD}}}
HTTP 200
[Captures]
login.token: jsonpath "$.access_token"
login.user_id: jsonpath "$.user.id"
[Asserts]
jsonpath "$.access_token" exists

GET https://api.example.com/users/{{login.user_id}}
Authorization: Bearer {{login.token}}
HTTP 200
[Asserts]
jsonpath "$.id" == {{login.user_id}}
jsonpath "$.role" isString

