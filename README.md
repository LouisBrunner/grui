# godot-grui

Reactive, declarative UI framework for Godot in Rust.

This workspace contains:

- [`godot-grui`](packages/grui) — the main crate: renderer, signals, components, and re-exports of the macros
- [`godot-grui-macros`](packages/grui-macros) — proc macros: `control!`, `#[component]`, `#[class]`
- [`godot-grui-example`](packages/example) — a Godot project demonstrating the API
