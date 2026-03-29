# Error Handling

## Error Handling

Reducers distinguish between two types of errors:

### Sender Errors

Errors caused by invalid client input. These are expected and should be handled gracefully.

Return an error:

```
#[reducer]
pub fn transfer_credits(
    ctx: &ReducerContext,
    to_user: u64,
    amount: u32
) -> Result<(), String> {
    let from_balance = ctx.db.users().id().find(ctx.sender.identity)
        .ok_or("User not found");

    if from_balance.credits < amount {
        return Err("Insufficient credits".to_string());
    }

    // ... perform transfer
    Ok(())
}
```

### Programmer Errors

Unexpected errors caused by bugs in module code. These should be fixed by the developer.

Panics or uncaught errors:

```
#[reducer]
pub fn process_data(ctx: &ReducerContext, data: Vec<u8>) -> Result<(), String> {
    // This panic indicates a bug
    assert!(data.len() > 0, "Unexpected empty data");

    // Uncaught Result indicates a bug
    let parsed = parse_data(&data).expect("Failed to parse data");

    // ...
    Ok(())
}
```

Programmer errors are logged and visible in your project dashboard. Consider setting up alerting to be notified when these occur.
