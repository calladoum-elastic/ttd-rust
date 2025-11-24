# `ttd-rust`

A Rust library for handling Microsoft Time Travel Debugging traces.

## Usage

### Replay

#### Get trace info

```rust
use ttd::replay::ReplayEngine;
use ttd::replay::events::EventType;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Print some info
  let si = engine.system_info()?;
  dbg!(&si);

  Ok(())
}
```


#### Navigate in a trace

```rust
use ttd::replay::ReplayEngine;
use ttd::replay::events::EventType;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Open a cursor to navigate the trace
  let mut cursor = engine.cursor()?;

  // Replay forward until the end of the trace
  let replay_result = cursor.replay_forward(None)?;
  assert_eq!(res.stop_reason, EventType::Process);

  // Jump to a position
  cursor.set_position( ReplayPosition{...});

  Ok(())
}
```


#### Accessing memory

```rust
use ttd::replay::ReplayEngine;
use ttd::replay::events::EventType;

fn main() -> Result<(), ttd::error::Error> {
  // Open a recording
  let mut engine = ReplayEngine::open(r"c:\path\to\my_trace.run")?;

  // Open a cursor to navigate the trace
  let cursor = engine.cursor()?;

  // Inspect memory state at a specific point
  let memory_dump = cursor.memory_at(123456)?;
  println!("Memory snapshot: {:?}", memory_dump);

  Ok(())
}
```