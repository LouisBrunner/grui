# grui

`grui` lets you build declarative, reactive user interfaces for Godot in Rust.
Inspired by React and Leptos, it combines a compact HTML-like syntax with the
Godot Rust bindings to render control-based UIs.

The API and DSL are inspired by [Leptos](https://www.leptos.dev/). You get
signals, effects and a `<For/>` element for
basic reactivity.

## Crates

The workspace contains two crates:

- `grui` – runtime types and Godot integration (`Renderer`, virtual nodes,
  reactive signals/effects, component trait, builders, etc.)
- `grui-macros` – provides the `control!` macro and `component` and
  `class` attributes. See `packages/grui-macros/README.md` for macro examples and tests.

Add them to your project via the workspace manifest or by depending on
`grui = { path = ".../packages/grui" }`.

## Quick start

```rust
use godot::prelude::*;
use grui::prelude::*;

#[component]
fn MenuButton(label: String, on_pressed: Callable) -> impl IntoControl {
    control!( <button on:pressed=on_pressed>{label}</button> )
}

#[component]
fn PauseMenu(title: String) -> impl IntoControl {
    let (count, set_count) = signal(0);

    Effect::new(|| {
        godot::godot_print!("Effect: count is {}", count.get());
    });

    let resume = Callable::from_fn(|| {
        godot::godot_print!("Resuming game!");
    });

    let quit = Callable::from_fn(|| {
        godot::godot_print!("Quitting game!");
    });

    control!(
        <vboxcontainer>
            <For each=|| (1..=3) key=|i| *i let(i)>
                <label text={format!("{} {}", title, i)} />
            </For>
            <For each=|| (0..count.get()) key=|i| *i let(i)>
                <label text=format!("Tick {}", i) />
            </For>
            <button on:pressed=Callable::from_fn(move || { set_count.update(|c| *c += 1); })>
                { format!("Clicks: {}", count.get()) }
            </button>
            <MenuButton label="Resume" on_pressed=resume />
            <MenuButton label="Quit" on_pressed=quit />
        </vboxcontainer>
    )
}

#[grui::prelude::class(PauseMenu)]
pub struct HudRoot {
    #[export]
    title: String,
}
```

### Reactivity

- `signal(initial)` returns `(ReadSignal<T>, WriteSignal<T>)` – call `set()` or
  `update()` to mutate, which marks the UI dirty. Renderer re-renders on next
  `process()`.
- `Effect::new(|| ...)` runs immediately and after each render triggered by any
  signal write.
- `for_each(iter, key, |item| ...)` builds a fragment from an iterator (simple
  `<For/>` substitute). Key currently unused but reserved for diffing.

### Lifecycle

Create a Godot class with `#[class(ComponentType)]` (from macros crate). The
renderer mounts once and re-renders when signals change.
