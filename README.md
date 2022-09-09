# opa

### OPA
OPA rust is an api that allows to manage authorizations. It is based on KRATOS Ory and integrates opa wasm.

To start the api follow the steps:

### RUN KRATOS (Before units-test)

```rust
## For start kratos
#Step 1:
make
#Step 2:
make kratos
#Step 3:
make start
#Step 4 (grep the id of an identity for your test curl):
make fake

## For stop kratos and clean repository
#Step 1:
make stop
#Step 2
make clean
```
### RUN OPA RUST AND CALL HIM

```rust
## For start opa rust api
#Step 1:
cargo build
#Step 2:
cargo run

## For call to CURL
#Step 1:
Get a identity ID to KRATOS SERVICE
#Step 2
curl -X POST -L http://127.0.0.1:8000 -H "Content-Type: application/json" -d '{"kratos": "<Kratos Identity ID>", "eval": "private_projects","resource": 222,"role":"admin","method": "get", "uri": "api/projects" }'

## For call to GRPCURL
#Step 1:
Get a identity ID to KRATOS SERVICE
#Step 2
grpcurl -plaintext -import-path ./src/proto -proto openpolicyagency.proto -d '<Kratos Identity ID>", "role":"Toutniquer2", "eval":"Toutniquer2", "uri":"Toutniquer2", "resource":1234, "method":"Toutniquer4"}' '[::]:3000' openpolicyagency.Opaproto/GetDecision//! ```
```

enjoy :)
