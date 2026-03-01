# Twine Models

Domain-specific models and model-building tools for [Twine](https://github.com/isentropic-dev/twine).

## Orientation

Start with `src/lib.rs` — the doc comments describe the crate layout, the relationship
between `models` and `support`, and how utility code progresses through the crate.

## Design Notes

- `ConstraintError` is an internal building block that may be deprecated.
  Prefer local error types with domain-meaningful messages in public APIs.

## Testing

Use `approx::assert_relative_eq!` for floating point comparisons.
