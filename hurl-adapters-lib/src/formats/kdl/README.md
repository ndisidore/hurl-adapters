# KDL Format for Hurl

This module translates [KDL](https://kdl.dev/) documents to [Hurl](https://hurl.dev/) format, providing a more structured and readable way to define HTTP request sequences.

## Basic Syntax

```kdl
METHOD "url" name="optional_step_name" {
    // Request configuration
    headers { ... }
    query { ... }
    form { ... }
    cookies { ... }
    basic-auth { ... }
    body <type> { ... }

    // Response expectations
    expect {
        status <code>
        captures { ... }
        asserts { ... }
    }
}
```

## Examples

### Simple GET Request

```kdl
GET "https://api.example.com/health"
```

**Output:**
```hurl
GET https://api.example.com/health
```

### GET with Status Check

```kdl
GET "https://api.example.com/users" {
    expect {
        status 200
    }
}
```

**Output:**
```hurl
GET https://api.example.com/users
HTTP 200
```

### GET with Query Parameters

```kdl
GET "https://api.example.com/search" {
    query {
        q "rust programming"
        limit "10"
        offset "0"
    }
    expect {
        status 200
    }
}
```

**Output:**
```hurl
GET https://api.example.com/search
[Query]
q: rust programming
limit: 10
offset: 0
HTTP 200
```

### POST with Headers and JSON Body

```kdl
POST "https://api.example.com/users" {
    headers {
        Content-Type "application/json"
        Accept "application/json"
    }
    body json {
        name "John Doe"
        email "john@example.com"
        age 30
    }
    expect {
        status 201
    }
}
```

**Output:**
```hurl
POST https://api.example.com/users
Content-Type: application/json
Accept: application/json
{
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30
}
HTTP 201
```

### Using Variables (Placeholders)

Variables use the `{{variable}}` syntax, compatible with Hurl's templating:

```kdl
POST "https://api.example.com/login" {
    headers {
        Content-Type "application/json"
    }
    body json {
        username "{{USERNAME}}"
        password "{{PASSWORD}}"
    }
    expect {
        status 200
    }
}
```

---

## Named Steps and Chaining

The real power comes from **named steps** that enable request chaining. When you name a step, its captured values are automatically prefixed with the step name.

### Basic Chaining: Login → Use Token

```kdl
// Step 1: Login and capture the token
POST "https://api.example.com/auth/login" name="login" {
    headers {
        Content-Type "application/json"
    }
    body json {
        username "testuser"
        password "secret123"
    }
    expect {
        status 200
        captures {
            token jsonpath "$.access_token"
        }
    }
}

// Step 2: Use the captured token
GET "https://api.example.com/profile" {
    headers {
        Authorization "Bearer {{login.token}}"
    }
    expect {
        status 200
    }
}
```

**Key points:**
- `name="login"` names the first step
- `token jsonpath "$.access_token"` captures the token as `login.token`
- `{{login.token}}` references the captured value in subsequent requests

### Multi-Step Workflow: Create → Read → Update → Delete

```kdl
// Create a new resource
POST "https://api.example.com/posts" name="create" {
    headers {
        Content-Type "application/json"
        Authorization "Bearer {{AUTH_TOKEN}}"
    }
    body json {
        title "My First Post"
        content "Hello, World!"
        published false
    }
    expect {
        status 201
        captures {
            id jsonpath "$.id"
            created_at jsonpath "$.created_at"
        }
        asserts {
            jsonpath "$.id" exists
            jsonpath "$.title" == "My First Post"
        }
    }
}

// Read the created resource
GET "https://api.example.com/posts/{{create.id}}" name="read" {
    headers {
        Authorization "Bearer {{AUTH_TOKEN}}"
    }
    expect {
        status 200
        asserts {
            jsonpath "$.id" == "{{create.id}}"
            jsonpath "$.title" == "My First Post"
        }
    }
}

// Update the resource
PUT "https://api.example.com/posts/{{create.id}}" name="update" {
    headers {
        Content-Type "application/json"
        Authorization "Bearer {{AUTH_TOKEN}}"
    }
    body json {
        title "My Updated Post"
        content "Updated content"
        published true
    }
    expect {
        status 200
        captures {
            updated_at jsonpath "$.updated_at"
        }
        asserts {
            jsonpath "$.title" == "My Updated Post"
            jsonpath "$.published" == true
        }
    }
}

// Delete the resource
DELETE "https://api.example.com/posts/{{create.id}}" {
    headers {
        Authorization "Bearer {{AUTH_TOKEN}}"
    }
    expect {
        status 204
    }
}

// Verify deletion
GET "https://api.example.com/posts/{{create.id}}" {
    headers {
        Authorization "Bearer {{AUTH_TOKEN}}"
    }
    expect {
        status 404
    }
}
```

### Complex Authentication Flow

```kdl
// Step 1: Get CSRF token from login page
GET "https://app.example.com/login" name="csrf" {
    expect {
        status 200
        captures {
            token regex "csrf_token\" value=\"([^\"]+)\""
        }
    }
}

// Step 2: Authenticate with CSRF token
POST "https://app.example.com/login" name="auth" {
    headers {
        Content-Type "application/x-www-form-urlencoded"
        X-CSRF-Token "{{csrf.token}}"
    }
    form {
        username "admin"
        password "{{ADMIN_PASSWORD}}"
        csrf_token "{{csrf.token}}"
    }
    expect {
        status 302
        captures {
            session cookie "session_id"
            redirect header "Location"
        }
    }
}

// Step 3: Follow redirect to dashboard
GET "{{auth.redirect}}" name="dashboard" {
    cookies {
        session_id "{{auth.session}}"
    }
    expect {
        status 200
        captures {
            user_id jsonpath "$.user.id"
            permissions jsonpath "$.user.permissions"
        }
        asserts {
            jsonpath "$.user.role" == "admin"
        }
    }
}

// Step 4: Access admin-only endpoint
GET "https://app.example.com/admin/users" {
    cookies {
        session_id "{{auth.session}}"
    }
    expect {
        status 200
        asserts {
            jsonpath "$.users" isCollection
            jsonpath "$.users[0].id" exists
        }
    }
}
```

---

## Request Sections Reference

### Headers

```kdl
headers {
    Content-Type "application/json"
    Authorization "Bearer {{token}}"
    X-Custom-Header "custom-value"
}
```

### Query Parameters

```kdl
query {
    page "1"
    per_page "20"
    sort "created_at"
    order "desc"
}
```

### Form Data

```kdl
form {
    username "john"
    password "secret"
    remember_me "true"
}
```

### Cookies

```kdl
cookies {
    session_id "abc123"
    preferences "dark_mode=true"
}
```

### Basic Authentication

```kdl
basic-auth {
    username "password"
}
```

---

## Body Types

### JSON

```kdl
body json {
    name "John"
    age 30
    active true
    tags "rust" "hurl" "testing"
    address {
        city "New York"
        zip "10001"
    }
}
```

### XML

```kdl
body xml "<user><name>John</name><email>john@example.com</email></user>"
```

### Plain Text

```kdl
body text "Plain text content here"
```

### GraphQL

```kdl
body graphql "query { user(id: 1) { name email } }" {
    variables {
        id "1"
    }
}
```

### File

```kdl
body file "path/to/file.bin"
```

### Base64

```kdl
body base64 "SGVsbG8gV29ybGQh"
```

### Hex

```kdl
body hex "48656c6c6f"
```

---

## Response Expectations

### Status Codes

```kdl
expect {
    status 200      // Specific status
    status "*"      // Any status (wildcard)
}
```

### Captures

Extract values from responses for use in subsequent requests:

```kdl
captures {
    // JSONPath
    token jsonpath "$.data.token"
    user_id jsonpath "$.user.id"
    first_item jsonpath "$.items[0].name"

    // XPath (for XML/HTML)
    title xpath "//title/text()"

    // Headers
    location header "Location"
    content_type header "Content-Type"

    // Cookies
    session cookie "session_id"

    // Regex
    csrf regex "token=\"([^\"]+)\""

    // Entire body
    raw_body body
}
```

### Assertions

Validate response data:

```kdl
asserts {
    // Equality
    jsonpath "$.status" == "success"
    jsonpath "$.count" == 42

    // Comparisons
    jsonpath "$.age" > 18
    jsonpath "$.price" <= 99.99

    // String operations
    jsonpath "$.name" startsWith "John"
    jsonpath "$.email" endsWith "@example.com"
    jsonpath "$.description" contains "important"
    jsonpath "$.code" matches "^[A-Z]{3}[0-9]{4}$"

    // Existence
    jsonpath "$.id" exists
    jsonpath "$.deleted_at" not exists

    // Type checks
    jsonpath "$.count" isInteger
    jsonpath "$.price" isFloat
    jsonpath "$.active" isBoolean
    jsonpath "$.name" isString
    jsonpath "$.items" isCollection
    jsonpath "$.tags" isEmpty

    // Status and duration
    status == 200
    duration < 1000
}
```

---

## Variable Naming Convention

| Context | Capture Definition | Variable Reference |
|---------|-------------------|-------------------|
| Named step "login" | `token jsonpath "$.token"` | `{{login.token}}` |
| Named step "api" | `user_id jsonpath "$.id"` | `{{api.user_id}}` |
| Unnamed step | `token jsonpath "$.token"` | `{{token}}` |
| Environment variable | N/A | `{{ENV_VAR}}` |

---

## Usage in Rust

```rust
use kdl::KdlDocument;
use hurl_adapters_lib::formats::kdl::{translate, translate_to_string, TranslationError};

fn main() -> Result<(), TranslationError> {
    let kdl_input = r#"
        GET "https://api.example.com/health" {
            expect {
                status 200
            }
        }
    "#;

    let doc: KdlDocument = kdl_input.parse()?;

    // Get Hurl AST
    let hurl_file = translate(&doc)?;

    // Or get Hurl string directly
    let hurl_string = translate_to_string(&doc)?;
    println!("{}", hurl_string);

    Ok(())
}
```
