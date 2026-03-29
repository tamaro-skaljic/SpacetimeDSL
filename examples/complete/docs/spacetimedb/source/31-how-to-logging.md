# Logging

SpacetimeDB provides logging capabilities for debugging and monitoring your modules. Log messages are private to the database owner and are not visible to clients.

## Writing Logs

Use the `log` crate to write logs from your reducers:

```
use spacetimedb::{reducer, ReducerContext};

#[reducer]
pub fn process_data(ctx: &ReducerContext, value: u32) -> Result<(), String> {
    log::info!("Processing data with value: {}", value);

    if value > 100 {
        log::warn!("Value {} exceeds threshold", value);
    }

    if value == 0 {
        log::error!("Invalid value: 0");
        return Err("Value cannot be zero".to_string());
    }

    log::debug!("Debug information: ctx.sender = {:?}", ctx.sender());

    Ok(())
}
```

Available log levels:

- `log::error!()` - Error messages
- `log::warn!()` - Warning messages
- `log::info!()` - Informational messages
- `log::debug!()` - Debug messages
- `log::trace!()` - Trace messages

## Best Practices

### Log Levels

Use appropriate log levels for different types of messages:

- **Error**: Use for actual errors that prevent operations from completing
- **Warn**: Use for potentially problematic situations that don't prevent execution
- **Info**: Use for important application events (user actions, state changes)
- **Debug**: Use for detailed diagnostic information useful during development
- **Trace**: Use for very detailed diagnostic information (typically disabled in production)

### Performance Considerations

- Logging has minimal overhead, but excessive logging can impact performance
- Avoid logging in tight loops or high-frequency operations
- Consider using debug/trace logs for verbose output that can be filtered in production

### Privacy and Security

- Logs are only visible to the database owner, not to clients
- Avoid logging sensitive information like passwords or authentication tokens
- Be mindful of personally identifiable information (PII) in logs

### Structured Logging

Use structured logging with key-value pairs for better log analysis:

```
use spacetimedb::log;

#[reducer]
pub fn transfer_credits(ctx: &ReducerContext, to_user: u64, amount: u32) -> Result<(), String> {
    log::info!(
        "Credit transfer: from={:?}, to={}, amount={}",
        ctx.sender(),
        to_user,
        amount
    );

    // ... transfer logic

    Ok(())
}
```

## Next Steps

- Learn about [Error Handling](/docs/functions/reducers/error-handling) in reducers
- Set up monitoring and alerting for your production databases
