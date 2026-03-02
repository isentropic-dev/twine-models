# Twine Models

Domain-specific models and model-building tools for [Twine](https://github.com/isentropic-dev/twine).

## What this crate provides

`twine-models` offers opinionated [`Model`][twine-core-model] implementations for engineering domains, along with the supporting utilities needed to build and compose them.

It is organized into two top-level modules:

- **`models`** — ready-to-use `Model` implementations
- **`support`** — utilities used by models, also available for downstream use

See the [crate docs](https://docs.rs/twine-models) for details on each module.

## Models

### Thermal

#### Heat exchangers (`models::thermal::hx`)

- **`Recuperator`** — a discretized counter-flow heat exchanger that solves for outlet conditions given a UA value and inlet streams

#### Tanks (`models::thermal::tank`)

- **`StratifiedTank`** — a multi-node stratified thermal storage tank with configurable ports, auxiliary heat, conductive losses, and buoyancy-driven mixing

## Feature flags

| Feature    | What it enables                                      | Default |
|------------|------------------------------------------------------|---------|
| `coolprop` | `support::thermo::model::CoolProp` (via `rfluids`)   | no      |
| `plot`     | `PlotObserver` integration via `twine-observers`     | no      |

Enable a feature in your `Cargo.toml`:

```toml
twine-models = { version = "0.1", features = ["coolprop"] }
```

## Examples

### Stratified tank simulation

Simulates five days of residential hot water tank operation with a thermostat-controlled heating element, a daily draw schedule, and an interactive time-series plot.

```sh
cargo run --example stratified_tank --features plot --release
```

## Utility code (`support`)

Modules in `support` are public because they're useful, but their APIs are not yet stable — breaking changes may occur. The lifecycle for utility code is:

1. **Model-specific** — starts private inside a model's `core` module
2. **Domain-specific** — moves to a domain support module when useful across models in a domain
3. **Crate-level** — moves to `support` when useful across domains
4. **Standalone** — may become its own crate if broadly useful and stable

## Twine ecosystem

| Crate | Description |
|-------|-------------|
| [`twine-core`](https://github.com/isentropic-dev/twine) | `Model` trait and core abstractions |
| [`twine-solvers`](https://github.com/isentropic-dev/twine) | ODE solvers (Euler, etc.) |
| [`twine-observers`](https://github.com/isentropic-dev/twine) | Observer utilities (plotting, logging) |

[twine-core-model]: https://docs.rs/twine-core/latest/twine_core/trait.Model.html
