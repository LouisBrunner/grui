# godot-grui-macros

Proc macros for [`godot-grui`](https://crates.io/crates/godot-grui).

You don't need to depend on this crate directly — `godot-grui` reexports everything via its prelude.

## Macros

- `control! { ... }` — JSX-like syntax for building UI trees from Godot control nodes and custom components
- `#[component]` — marks a function as a reactive component
- `#[class(...)]` — wires a Godot class to a root component, handling mounting and prop forwarding
