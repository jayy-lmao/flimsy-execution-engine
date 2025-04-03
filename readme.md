## Why

inconsistent, fault-intolerant, and unresilient execution of code.

This is a unreliable execution engine. Consider using in production if you want dataloss, low throughput, and poor support.

## Running locally

```sh
cargo run
```

then just mess with the code I guess

## TODO

- [x] Run activities from a Workflow.
- [x] Allow manual re-running of activities/workflows.
    - [ ] Handle re-run if same activity name is used multiple times (event order).
    - [ ] Allow specifying which checkpoint to run from.
- [ ] Allow failed activities to retry n times.
- [ ] Timeouts
- [ ] Persist in a DB.
