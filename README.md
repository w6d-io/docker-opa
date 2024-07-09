# opa

### OPA
OPA rust is an api that allows to manage authorizations. It is based on KRATOS Ory.

To start the api follow the steps:

### WRITE A VALIDE REGO FILE

The rego file conformity can be tested with the [Regorus playground](https://anakrish.github.io/regorus-playground/) and the official [rego playground](https://play.openpolicyagent.org/).

### RUN OPA RUST AND CALL HIM

```bash
## For start opa rust api
#Step 1:
cargo build
#Step 2:
cargo run
#Step 3:
curl -X POST -L http://127.0.0.1:8000 -H "Content-Type: application/json" -d '{"input": {"resource": "222","role":"admin",\
"method": "get", "uri": "<uri/to/the/caller>" }, "data": {<kratos identity json>}}'
```
enjoy :)

