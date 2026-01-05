POST https://api.example.com/posts
Content-Type: application/json
Authorization: Bearer {{AUTH_TOKEN}}
{
    "title": "My First Post",
    "content": "Hello, World!"
}
HTTP 201
[Captures]
create_id: jsonpath "$.id"
[Asserts]
jsonpath "$.id" exists
jsonpath "$.title" == "My First Post"

GET https://api.example.com/posts/{{create_id}}
Authorization: Bearer {{AUTH_TOKEN}}
HTTP 200
[Asserts]
jsonpath "$.title" == "My First Post"

DELETE https://api.example.com/posts/{{create_id}}
Authorization: Bearer {{AUTH_TOKEN}}
HTTP 204

GET https://api.example.com/posts/{{create_id}}
Authorization: Bearer {{AUTH_TOKEN}}
HTTP 404

