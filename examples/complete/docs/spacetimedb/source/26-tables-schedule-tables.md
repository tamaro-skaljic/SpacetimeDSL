# Schedule Tables

Tables can trigger [reducers](/docs/functions/reducers) or [procedures](/docs/functions/procedures) at specific times by including a special scheduling column. This allows you to schedule future actions like sending reminders, expiring items, or running periodic maintenance tasks.

> **Scheduling Procedures**
>
> Procedures use the same scheduling pattern as reducers. Simply reference the procedure name in the `scheduled` attribute. This is particularly useful when you need scheduled tasks that make HTTP requests or perform other side effects. See [Scheduling Procedures](/docs/functions/reducers#scheduling-procedures) for an example.

## Defining a Schedule Table

> **Why "scheduled" in the code?**
>
> The table attribute uses `scheduled` (with a "d") because it refers to the **scheduled reducer** - the function that will be scheduled for execution. The table itself is a "schedule table" that stores schedules, while the reducer it triggers is a "scheduled reducer".

```
#[spacetimedb::table(accessor = reminder_schedule, scheduled(send_reminder))]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    id: u64,
    user_id: u32,
    message: String,
    scheduled_at: ScheduleAt,
}

#[spacetimedb::reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) -> Result<(), String> {
    // Process the scheduled reminder
    Ok(())
}
```

## Inserting Schedules

To schedule an action, insert a row into the schedule table with a `scheduled_at` value. You can schedule actions to run:

- **At intervals** - Execute repeatedly at fixed time intervals (e.g., every 5 seconds)
- **At specific times** - Execute once at an absolute timestamp

### Scheduling at Intervals

Use intervals for periodic tasks like game ticks, heartbeats, or recurring maintenance:

```
use spacetimedb::{ScheduleAt, ReducerContext};
use std::time::Duration;

#[spacetimedb::reducer]
fn schedule_periodic_tasks(ctx: &ReducerContext) {
    // Schedule to run every 5 seconds
    ctx.db.reminder().insert(Reminder {
        id: 0,
        message: "Check for updates".to_string(),
        scheduled_at: ScheduleAt::Interval(Duration::from_secs(5).into()),
    });

    // Schedule to run every 100 milliseconds
    ctx.db.reminder().insert(Reminder {
        id: 0,
        message: "Game tick".to_string(),
        scheduled_at: ScheduleAt::Interval(Duration::from_millis(100).into()),
    });
}
```

### Scheduling at Specific Times

Use specific times for one-shot actions like sending a reminder at a particular moment or expiring content:

```
use spacetimedb::{ScheduleAt, ReducerContext};
use std::time::Duration;

#[spacetimedb::reducer]
fn schedule_timed_tasks(ctx: &ReducerContext) {
    // Schedule for 10 seconds from now
    let ten_seconds_from_now = ctx.timestamp + Duration::from_secs(10);
    ctx.db.reminder().insert(Reminder {
        id: 0,
        message: "Your auction has ended".to_string(),
        scheduled_at: ScheduleAt::Time(ten_seconds_from_now),
    });

    // Schedule for immediate execution (current timestamp)
    ctx.db.reminder().insert(Reminder {
        id: 0,
        message: "Process now".to_string(),
        scheduled_at: ScheduleAt::Time(ctx.timestamp.clone()),
    });
}
```

## How It Works

1. **Insert a row** with a `ScheduleAt` value
2. **SpacetimeDB monitors** the schedule table
3. **When the time arrives**, the specified reducer/procedure is automatically called with the row as a parameter
4. **The row is typically deleted** or updated by the reducer after processing

## Security Considerations

> **Scheduled Reducers Are Callable by Clients**
>
> Scheduled reducers are normal reducers that can also be invoked by external clients. If a scheduled reducer should only execute via the scheduler, add authentication checks.

```
#[spacetimedb::reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) -> Result<(), String> {
    if !ctx.sender_auth().is_internal() {
        return Err("This reducer can only be called by the scheduler".to_string());
    }
    // Process the scheduled reminder
    Ok(())
}
```

## Use Cases

- **Reminders and notifications** - Schedule messages to be sent at specific times
- **Expiring content** - Automatically remove or archive old data
- **Delayed actions** - Queue up actions to execute after a delay
- **Periodic tasks** - Schedule repeating maintenance or cleanup operations
- **Game mechanics** - Timer-based gameplay events (building completion, energy regeneration, etc.)

## Next Steps

- Learn about [Reducers](/docs/functions/reducers) to handle scheduled actions
- Explore [Procedures](/docs/functions/procedures) for scheduled execution patterns
