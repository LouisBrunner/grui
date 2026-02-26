# grui

`grui` lets you build declarative, reactive user interfaces for Godot in Rust.
Inspired by React and Leptos, it combines a compact HTML-like syntax with the Godot Rust bindings to render control-based UIs.

The API and DSL are inspired by [Leptos](https://www.leptos.dev/). It uses the same library for reactivity and thus provides similar features. This means you get signals (not to be confused with the Godot Signals), effects and a `<For/>` element for example.

## Crates

The workspace contains two crates:

- `grui`: contains the `Renderer` and all the reactive elements, it provides a `prelude` which also imports the `grui-macros`.
- `grui-macros`: provides the `control!` macro and `#[component]` and `#[class]` attributes.

## Quick start

```rust
use godot::prelude::*;
use grui::prelude::*;

// easily create custom component
#[component]
fn MenuButton(label: String, on_pressed: Callable) -> impl IntoControl {
    control! { <button on:pressed=on_pressed text=label /> }
}

// a top-level component is the same as any other
#[component]
fn PauseMenu(title: String) -> impl IntoControl {
    let (count, set_count) = signal(0);

    Effect::new(move || {
        godot_print!("Effect: count is {}", count.get());
    });

    let resume = SignalCallable::new(|_| {
        godot_print!("Resuming game!");
    });

    let quit = SignalCallable::new(|_| {
        godot_print!("Quitting game!");
    });

    control! {
        // built-in tags
        <vboxcontainer>
            // static iteration
            {
              (1..=3).map(|i| {
                  control! { <label text=format!("{} {}", title, i) /> }
              }).collect::<Vec<_>>()
            }
            // dynamic iteration
            <For each=|| (0..count.get()) key=|i| *i let(i)>
                <label text=format!("Tick {}", i) />
            </For>
            // event handling
            <button on:pressed=SignalCallable::new(move |_| {
                godot_print!("Button pressed! (count: {})", count.get());
                set_count.update(|c| *c += 1);
              })
              text=|| format!("Clicks: {}", count.get()) />
            // conditions
            {move || if count.get() > 3 {
              control!{ <label text="STOP!" /> }.into_any()
            } else {
              control!{ <button text="Keep pressing!" /> }.into_any()
            }}
            <Show
              when=|| {count.get() > 3}
              fallback=|| control!{ <button text="Keep pressing!" /> }
            >
                <label text="STOP!" />
            </Show>
            // custom component usage
            <MenuButton label="Resume" on_pressed=resume />
            <MenuButton label="Quit" on_pressed=quit />
        </vboxcontainer>
    }
}

// this struct can now be used directly in the Godot Editor
#[grui::prelude::class(root=PauseMenu)]
pub struct HUDRoot {
    // properties are given as props to the root component
    #[export]
    title: String,
}
```

### Reactivity

- `signal(initial)` returns `(ReadSignal<T>, WriteSignal<T>)`, call `set()` or `update()` to mutate, which marks the UI dirty.
- `Effect::new(|| ...)` runs immediately and after each render triggered by any state write.
- All the other functions provided by `reactive_graph` (see Leptos' documentation for more details).
- `<For/>` and `<ForEnumerate />` for dynamic amount of entries.
- `<Show />` for efficient conditions.

## Missing

- [ ] Resolve warnings for ForEnumerate
- [ ] Redo <Show /> Children
- [ ] Optional props
- [ ] Themes & Override
- [ ] Forms
- [ ] Connect to external Godot signals / Allow root class to change props
- [ ] TestRenderer + allow changes
- [ ] Better fallback macros for invalid syntax
- [ ] Statically typed props/signals
- [ ] Hydration
- [ ] Preview in Godot Editor

## Acknowledgments

- Leptos: which deeply influenced this project. The `grui` API is a slightly slimmed down version of the Leptops one. We also use their brilliant `reactive_graph` crate. Some of the structures/types are pretty much the same. Thank you for your amazing work!
