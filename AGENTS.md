# Twine Models

Domain-specific models and model-building tools for [Twine](https://github.com/isentropic-dev/twine).

## Crate Layout

- `src/models/`: `twine_core::Model` implementations, organized by domain (e.g., `thermal`)
- `src/support/`: General-purpose engineering utilities — thermodynamics, heat exchangers,
  turbomachinery, control, scheduling, units

## `support` vs `models`

`models` contains `twine_core::Model` implementations where the solver/observer machinery
earns its keep — types that wrap a computational `core` and expose `Parameters`, `Inputs`,
`Outputs`, and `State` associated types.

`support` is useful engineering code that doesn't need the `Model` abstraction —
thermodynamic state calculations, ε-NTU relations, isentropic efficiency types, schedules,
control logic. It's part of the public API but not stable; breaking changes may occur.

## Model Structure

Each model module has an internal `core` submodule where computation lives.
The `Model` implementation is a thin adapter that delegates to `core`.
A single `core` can be exposed through multiple adapters.

`core` is an implementation detail — not part of the public API.

## Utility Code Lifecycle

Utility code progresses as needs emerge:

1. **Model-specific** — starts in a model's internal `core`
2. **Domain-specific** — moves to a domain-level support module when useful across models
3. **Crate-level** — moves to `support` when useful across domains
4. **Standalone** — may become its own crate if broadly useful and stable

## API Design Notes

- Domain types (e.g., `IsentropicEfficiency`, `Deadband`) should validate with
  local error types carrying domain-meaningful messages.
  `ConstraintError` is an internal building block — don't expose it in public APIs.

## Testing

Use `approx::assert_relative_eq!` for floating point comparisons.
