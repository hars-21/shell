# reqsh (Rust Learning Project)

A lightweight interactive shell to run and manage API checks faster.

## Why

I was repeating the same curl commands for local and staging APIs.
So I added simple built-in commands to save and run checks quickly.

## Built-in check commands

- `savecheck <name> <command...>`
- `runcheck <name>`
- `listchecks`
- `delcheck <name>`

Checks are saved in `checks.txt` as `name|command`.

## Example

```bash
savecheck health curl -s http://localhost:3000/health
savecheck users curl -s http://localhost:3000/users
listchecks
runcheck health
delcheck users
```

## Run

```bash
cargo run
```
