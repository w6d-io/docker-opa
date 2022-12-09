# opa

### OPA
OPA rust is an api that allows to manage authorizations. It is based on KRATOS Ory and integrates opa wasm.

To start the api follow the steps:

### RUN OPA RUST AND CALL HIM

```rust
## For start opa rust api
#Step 1:
cargo build
#Step 2:
cargo run
curl -X POST -L http://127.0.0.1:8000 -H "Content-Type: application/json" -d '{"input": {"resource": "222","role":"admin",\
"method": "get", "uri": "<uri/to/the/caller>" }, "data": {<kratos identity json>}}'

enjoy :)
