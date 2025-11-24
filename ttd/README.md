# `ttd-rust`

A Rust library for handling Microsoft Time Travel Debugging traces.

The purpose is to provide safe, ergonomic Rust bindings and higher-level helpers for the Microsoft TTD SDK so you can load traces, inspect recorded state (memory, threads, modules, registers), navigate forward/backward through execution, and register watchpoints or event observers. This crate interacts with tThe lower level crate `ttd_sys` which handles the FFI bindings and expose them to Rust as more friendly types (`ReplayEngine`, `ReplayCursor`, `ReplayPosition`, `ReplayModule`, `SystemInfo`, etc.) which Rust  manages ownership, lifetimes on.

Key features already implemented are:
 * Load and open TTD traces from disk and query trace metadata (system info, process id, modules, threads).
 * Create borrowed cursors to navigate recorded execution forward and backward, seek to positions, and read memory/register state as captured.
 * Register memory-based and position-based watchpoints to observe when particular addresses or positions are reached during replay.
 * Enumerate recorded events (module load/unload, exceptions, and other SDK event types) with both high-level Rust types and raw FFI event records.
* Rely on Rust lifetime management to ensure no corruption happens at the lower level.

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
  dbg!(engine.system_info);

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

  // Replay forward until the end of the trace
  let replay_result = engine.replay_forward(None)?;
  assert_eq!(res.stop_reason, EventType::Process);

  // Jump to a position
  engine.set_position( ReplayPosition{...});

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

  // Inspect memory state at a specific point
  let memory_dump = session.memory_at(123456)?;
  println!("Memory snapshot: {:?}", memory_dump);

  Ok(())
}
```