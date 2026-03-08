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

// easily create custom component (with automatic into conversion of prop)
#[component]
fn MenuButton(#[prop(into)] label: String, on_pressed: Callable) -> impl IntoControl {
    control! { <button on:pressed=on_pressed text=label.clone() /> }
}

// a top-level component is the same as any other (with optional prop)
#[component]
fn PauseMenu(#[prop(optional) title: String) -> impl IntoControl {
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
        // built-in tags with attributes
        <vboxcontainer anchor_right=1.0 anchor_bottom=1.0>
            // static iteration
            {
              (1..=3).map(|i| {
                  control! { <label text=format!("{} {}", title.unwrap_or("test".to_string()), i) /> }
              }).collect_control()
            }
            // dynamic iteration (different syntaxes)
            <For each=|| (0..count.get()) key=|i| *i let(i)>
                <label text=format!("Tick {}", i) />
            </For>
            // event handling
            <button on:pressed=SignalCallable::new(move |_| {
                godot_print!("Button pressed!");
                set_count.update(|c| *c += 1);
              })
              text=|| format!("Clicks: {}", count.get()) />
            // conditions
            {move || if count.get() > 3 { // inefficient
              control!{ <label text="STOP!" /> }.into_any()
            } else {
              control!{ <button text="Keep pressing!" /> }.into_any()
            }}
            <Show // performant!
              when=|| {count.get() > 3}
              fallback=|| control!{ <button text="Keep pressing!" /> } // optional
            >
                // theme override
                <label theme_override_font_sizes:font_size=30 text="STOP!" />
            </Show>
            // custom component usage
            <MenuButton label="Resume" on_pressed=resume />
            <MenuButton label="Quit" on_pressed=quit />
            // generic: html-like div equivalent
            <Generic background=Color::RED>
              <label text="HTML-like" />
              <Generic background=Color::BLUE margin=5.0 padding=10.0 display=Display::Horizontal(SubDisplay::Linear)>
                <label text="Sub 1" />
                <label text="Sub 2" />
              </Generic>
            </Generic>
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

### Testing

The crate provides a `TestRenderer` which can be used to test your components, e.g.

```rust
#[test]
fn with_simple() {
    TestRenderer::mount(control! { <Simple a=42 b="dauphin" /> }, |renderer| {
        assert_eq!(
            renderer
                .get_root()
                .snapshot()
                .expect("snapshot to be correct"),
            r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"a: 42, b: dauphin\""}}]}"#
        );
    });
}
```

You can also trigger Godot signals to simulate user interactions.

```rust
#[tokio::test]
async fn with_reactive() {
    TestRenderer::mount_async(control! { <Reactive /> }, |renderer| async move {
        assert_eq!(
            renderer
                .get_root()
                .snapshot()
                .expect("snapshot to be correct"),
            r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"0\""}},{"type":"Button","props":{"text":"\"+\""},"signals":["click"]}]}"#
        );
        renderer
            .get_root()
            .select_by_indices("1")
            .expect("to find button")
            .emit_signal("click", &[]);

        wait_for_async_changes();

        assert_eq!(
            renderer
                .get_root()
                .snapshot()
                .expect("snapshot to be correct"),
            r#"{"type":"Root","children":[{"type":"Label","props":{"text":"\"1\""}},{"type":"Button","props":{"text":"\"+\""},"signals":["click"]}]}"#
        );
    }).await;
}
```

## Missing

- [ ] Better fallback macros for invalid syntax
- [ ] Statically typed props/signals
- [ ] Bind?
- [ ] Hydration
- [ ] Preview in Godot Editor

## Acknowledgments

- Leptos: which deeply influenced this project. The `grui` API is a slightly slimmed down version of the Leptops one. We also use their brilliant `reactive_graph` crate. Some of the structures/types are pretty much the same. Thank you for your amazing work!
