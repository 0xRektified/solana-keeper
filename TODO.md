# TODO

## Architecture Refactor
- [ ] Separate program-specific logic from generic Solana infrastructure
  - Move `ResolveExecutor` to `src/implementations/resolve/executor.rs`
  - Move `TaskAccount`, `ConfigAccount`, `EpochAccount` to `src/implementations/resolve/state.rs`
  - Move `TimestampTrigger` to `src/implementations/resolve/trigger.rs` (if program-specific)
  - Create generic `SolanaExecutor` in `src/solana/executor.rs` for reusable transaction logic
  - Keep `src/core/` as pure trait definitions
  - Keep `src/solana/` as Solana platform infrastructure
  - Use `src/implementations/` for program-specific keeper logic

### Benefits
- Makes keeper reusable for other programs/instructions
- Clear separation of concerns
- Easy to add new keeper implementations

### Target Structure
```
src/
├── core/
│   ├── executor.rs      # Generic Executor trait
│   ├── trigger.rs       # Generic Trigger trait
│   └── watcher.rs       # Generic Watcher
├── solana/
│   ├── executor.rs      # Generic Solana transaction executor
│   ├── trigger.rs       # Generic timestamp/condition triggers
│   └── state.rs         # Generic state management
└── implementations/     # Program-specific logic
    └── resolve/
        ├── executor.rs  # ResolveExecutor with custom instruction building
        ├── state.rs     # TaskAccount, ConfigAccount, etc.
        └── trigger.rs   # Custom trigger logic if needed
```
