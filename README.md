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
            <button on:pressed=SignalCallable::new(move |_| { set_count.update(|c| *c += 1); })
              text=format!("Clicks: {}", count.get()) />
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

- `signal(initial)` returns `(ReadSignal<T>, WriteSignal<T>)` – call `set()` or `update()` to mutate, which marks the UI dirty.
- `Effect::new(|| ...)` runs immediately and after each render triggered by any state write.
- `for_each(iter, key, |item| ...)` builds a fragment from an iterator (simple `<For/>` substitute).
- create a Godot class with `#[class(root=ComponentType)]`. The renderer mounts once and re-renders when states change.

# Missing

- [x] Support all classes
- [ ] Conditions
- [ ] `state`
- [ ] `Effect`
- [ ] `for_each` + `<ForEnumerate />`
- [ ] `<Show />`
- [ ] `<ErrorBoundary/>`
- [ ] `Children` / `ChildrenFn` / `ChildrenFragment`
- [ ] Better form integration (bind?)
- [ ] Themes & Override
- [ ] Statically typed props/signals
- [ ] Memo?
- [ ] Context?
- [ ] Preview in Godot Editor
- [ ] Optional props
- [ ] Better fallback macros for invalid syntax
