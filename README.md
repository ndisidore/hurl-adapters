# hurl-adapters

Convert HTTP test definitions from KDL format to [Hurl](https://hurl.dev/).

## Packages

| Crate | Description |
|-------|-------------|
| `hurl-adapters-lib` | Core library for KDL to Hurl translation |
| `hurl-adapters-cli` | CLI tool (`hurl-adapt`) |

## Installation

```bash
cargo install --path hurl-adapters-cli
```

## Usage

Define your HTTP tests in KDL:

```kdl
// api-test.kdl
POST "https://api.example.com/users" name="create_user" {
    headers {
        Content-Type "application/json"
    }
    body json {
        name "Alice"
        email "alice@example.com"
    }
    expect {
        status 201
        captures {
            user_id jsonpath "$.id"
        }
    }
}

GET "https://api.example.com/users/{{create_user.user_id}}" {
    expect {
        status 200
        asserts {
            jsonpath "$.name" == "Alice"
        }
    }
}
```

Convert to Hurl and run:

```bash
# Convert and pipe directly to hurl
hurl-adapt api-test.kdl | hurl

# Or save to file first
hurl-adapt api-test.kdl -o api-test.hurl
hurl api-test.hurl

# Validate KDL syntax without output
hurl-adapt --check api-test.kdl
```

## CLI Options

```
hurl-adapt [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Input file (reads from stdin if omitted)

Options:
  -f, --format <FORMAT>  Input format [default: kdl]
  -o, --output <OUTPUT>  Output file (writes to stdout if omitted)
  -c, --check            Validate without producing output
  -q, --quiet            Suppress non-error output
```
