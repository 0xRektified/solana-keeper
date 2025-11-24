# A Pluggable On-Chain Task Scheduler

Sentinel is a lightweight Rust service designed to watch on-chain state and automatically trigger custom actions when conditions are met.

The first use-case is Solana: monitoring a program account that contains start_at / end_at timestamps, and calling a resolve instruction once end_at is reached. But Sentinel is designed to be generic, so developers can plug in their own logic, triggers, and execution handlers.

# Why Solana-seeker?

Most on-chain smart contracts cannot execute themselves. They need an off-chain “keeper” to:

- Watch account state
- Detect when a condition becomes true
- Submit the correct transaction at the right time

# Solana-seeker fills that gap.

## Core Concepts

Solana-seeker is built around three simple ideas:

### Watcher

A loop that observes external state (e.g., Solana account data).
It determines when a trigger condition has been met.

Examples:

end_at timestamp reached

status changed to Finished

balance exceeds threshold

### Trigger

A logical rule that returns true when it’s time to act.
This will be pluggable so each user can define their own rule.

Example trigger signature:

fn should_trigger(state: &AccountState) -> bool;

### Executor

The component responsible for performing the action once triggered.

Example:

fn execute(state: &AccountState) -> Result<(), Error>;


In the Solana case, this sends a resolve transaction.

## Architecture Overview

```bash
src/
 ├─ core/
 │   ├─ watcher.rs      // polling loop, timing, retries
 │   ├─ trigger.rs      // trigger trait
 │   └─ executor.rs     // executor trait
 ├─ solana/
 │   ├─ state.rs        // deserialize account state
 │   ├─ trigger.rs      // "end_at reached" implementation
 │   └─ executor.rs     // calls resolve instruction
 └─ main.rs             // wires up the chosen implementation
```

Traits Make It Pluggable

```rust
pub trait Trigger<T> {
    fn should_trigger(&self, state: &T) -> bool;
}

pub trait Executor<T> {
    fn execute(&self, state: &T) -> Result<(), Box<dyn std::error::Error>>;
}
```

Anyone can implement these traits for their own use case.

## Example Flow

Loads the target account from Solana RPC.

It parses the account into a type (AccountState).

It evaluates the trigger.
-> if should_trigger(state) == true

The executor runs — e.g., sending resolve.

Logs, Sleeps, and Repeats.

## Configuration

Future config options may include:

RPC endpoint

polling frequency

target account pubkey

program ID

retries & backoff

executor wallet keypair

## Goals

Simple to understand

Generic & extensible

Safe & robust

Zero magic / clear ownership

Great learning project for:

async Rust

Solana client programming

modular architecture

traits & abstraction

## Non-Goals (for now)

Running as a distributed cluster

Off-chain scheduling marketplace

Full cron syntax

UI/dashboard

These can come later if necessary.

## License

MIT — free for anyone to build on.