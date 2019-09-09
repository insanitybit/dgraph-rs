# dgraph-rs
A DGraph client for Rust

Supports Dgraph 1.1.x

Requires rustc 1.39 or higher for async await support.

This client is under development and does not support a number of DGraph features, such as
authentication.

### Creating a client

```rust
fn local_dgraph_client() -> DgraphClient {
    let addr = "localhost";
    let port = 9080;

    let client = api_grpc::DgraphClient::with_client(
        Arc::new(
            Client::new_plain(addr, port, ClientConf {
                ..Default::default()
            }).expect("Failed to initialize client stub")
        )
    );

    DgraphClient::new(vec![client])
}
```
    
### Query
```rust
fn main() {
    let dg = local_dgraph_client();
    let mut txn = dg.new_txn();
    let query_res: Value = txn.query(r#"
        {
          q0(func: has(node_key)) {
            uid
          }
        }
    "#)
        .await
        .map(|res| serde_json::from_slice(&res.json))
        .expect("Dgraph query failed")
        .expect("Json deserialize failed");
}
```

### Mutate
```rust
fn main() {
    let dg = local_dgraph_client();
    
    let mu = api::Mutation {
        set_nquads: br#"
        uid(p) <node_key> "{453120d4-5c9f-43f6-b7af-28b376b3a993}" .
        uid(p) <process_name> "foo.exe" ."#.to_vec(),
        ..Default::default()
    };
    
    let txn = dg.new_txn();
    txn.mutate(mu)
        .await
        .expect("Request to dgraph failed");
}
```

### Upsert
```rust
fn main() {
    let dg = local_dgraph_client();
    
    let query = r#"
        {
          p as var(func: eq(node_key, "{453120d4-5c9f-43f6-b7af-28b376b3a993}"))
        }
        "#;
    
    let mu = api::Mutation {
        set_nquads: br#"
        uid(p) <node_key> "{453120d4-5c9f-43f6-b7af-28b376b3a993}" .
        uid(p) <process_name> "foo.exe" ."#.to_vec(),
        ..Default::default()
    };
    
    let txn = dg.new_txn();
    txn.upsert(query, mu)
        .await
        .expect("Request to dgraph failed");
}
```

### Running tests
Tests require a local dgraph server, version 1.1.0 or higher.