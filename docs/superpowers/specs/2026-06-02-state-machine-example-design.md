# State Machine Example Design Spec

**Date:** 2026-06-02
**Issue:** #13 - Add state machine / statechart example
**Status:** Approved

---

## Overview

Add a compact runnable Rust example showing Minigraf as the persistence, audit, and guard-evaluation layer for a durable state machine. The scenario is an order workflow:

`:awaiting-payment -> :paid -> :shipped`

The example stays aligned with the existing examples: a small scenario function, a tiny binary under `examples/`, an exact-output integration test, and a README section.

## Behavior

The scenario initializes an order in `:awaiting-payment` and stores legal transitions as Minigraf facts. A Datalog guard query checks whether an event is legal from the order's current state.

The example demonstrates:

- Current state as a fact: `[:order-42 :fsm/state :awaiting-payment]`
- Transition topology as data facts using `:fsm/from`, `:fsm/event`, and `:fsm/to`
- Guard evaluation by querying the transition table and current state
- One legal transition for `:payment-received`
- One rejected illegal transition for `:ship` before payment
- Atomic state change using `begin_write()`, `retract`, `transact`, and `commit()`
- Transaction-time replay with `:as-of 1` to show the prior state

## Files

- `src/scenarios.rs`: add `state_machine()` and a small local helper for guard result row detection.
- `examples/state_machine.rs`: runnable example binary mirroring existing examples.
- `tests/scenarios.rs`: exact output test for the new scenario.
- `README.md`: sibling scenario section with run command and expected output.

## Output

The output remains three lines:

```text
State machine: accepted payment by querying transition facts as the guard.
State machine: rejected shipping from awaiting-payment before the transition.
State machine: replayed transaction history to explain the prior state.
```

## Testing

`tests/scenarios.rs` pins the output. The scenario itself executes all required Minigraf operations, and returns an error if guard queries do not have the expected legal/illegal result shapes.
