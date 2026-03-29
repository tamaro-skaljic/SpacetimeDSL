# Reject Client Connections

SpacetimeDB provides a way to disconnect a client during a client connection attempt.

In Rust, if we returned and error (or a panic) during the `client_connected` reducer, the client will be disconnected.

Here is a simple example where the server module throws an error for all incoming client connections.

```
#[reducer(client_connected)]
pub fn client_connected(_ctx: &ReducerContext) -> Result<(), String> {
    let client_is_rejected = true;
    if client_is_rejected {
        Err("The client connection was rejected. With our current code logic, all clients will be rejected.".to_string())
    } else {
        Ok(())
    }
}
```

Regardless of the client type, from the rust server's perspective, the client will be disconnected and the server module's logs will contain an entry reading:
`ERROR: : The client connection was rejected. With our current code logic, all clients will be rejected.`
