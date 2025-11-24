A Pluggable On-Chain Task Scheduler

Sentinel is a lightweight Rust service designed to watch on-chain state and automatically trigger custom actions when conditions are met.

The first use-case is Solana: monitoring a program account that contains start_at / end_at timestamps, and calling a resolve instruction once end_at is reached. But Sentinel is designed to be generic, so developers can plug in their own logic, triggers, and execution handlers.

🚀 Why Sentinel?

Most on-chain smart contracts cannot execute themselves. They need an off-chain “keeper” to:

✅ Watch account state
✅ Detect when a condition becomes true
✅ Submit the correct transaction at the right time

Sentinel fills that gap.

✅ Core Concepts

Sentinel is built around three simple ideas:

1️⃣ Watcher

A loop that observes external state (e.g., Solana account data).
It determines when a trigger condition has been met.

Examples:

end_at timestamp reached

status changed to Finished

balance exceeds threshold

2️⃣ Trigger

A logical rule that returns true when it’s time to act.
This will be pluggable so each user can define their own rule.

Example trigger signature:

fn should_trigger(state: &AccountState) -> bool;

3️⃣ Executor

The component responsible for performing the action once triggered.

Example:

fn execute(state: &AccountState) -> Result<(), Error>;


In the Solana case, this sends a resolve transaction.

🧩 Architecture Overview
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

Traits Make It Pluggable
pub trait Trigger<T> {
    fn should_trigger(&self, state: &T) -> bool;
}

pub trait Executor<T> {
    fn execute(&self, state: &T) -> Result<(), Box<dyn std::error::Error>>;
}


Anyone can implement these traits for their own use case.

💡 Example Flow

Sentinel loads the target account from Solana RPC.

It parses the account into a type (AccountState).

It evaluates the trigger.
✅ if should_trigger(state) == true

The executor runs — e.g., sending resolve.

Sentinel logs, sleeps, and repeats.

🔧 Configuration

Future config options may include:

RPC endpoint

polling frequency

target account pubkey

program ID

retries & backoff

executor wallet keypair

✅ Goals

Simple to understand

Generic & extensible

Safe & robust

Zero magic / clear ownership

Great learning project for:

async Rust

Solana client programming

modular architecture

traits & abstraction

❌ Non-Goals (for now)

Running as a distributed cluster

Off-chain scheduling marketplace

Full cron syntax

UI/dashboard

These can come later if the core project succeeds.

🏁 Getting Started
cargo new sentinel
cd sentinel


Then:

Implement the Trigger and Executor traits.

Write a simple watcher loop using tokio::time::sleep.

Connect to Solana RPC.

Poll the account & test triggering.

🌱 Roadmap

 MVP watcher loop

 Solana RPC integration

 Deserialize account state

 Timestamp trigger

 resolve instruction executor

 Logging & error handling

 Config file

 Plugin system via traits

🧭 Why this is a great learning project

You’ll learn:

✅ Rust async
✅ Traits & modular design
✅ Ownership & lifetimes in a real system
✅ Solana account reading
✅ Transaction submission
✅ Error handling & retries

It’s real, useful, and small enough to finish.

📛 Name Ideas (pick one)

Sentinel ✅ (recommended)

Watchtower

TriggerCore

ChainKeeper

TimeCall

Clockwork-Lite

OnChain Scheduler

“Sentinel” fits the vibe: a silent guard reacting when conditions are met.

✅ License

MIT — free for anyone to build on.