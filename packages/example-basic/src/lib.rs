use godot::prelude::*;
use grui::prelude::*;
use std::fmt::{Debug, Display};

#[component]
fn MenuButton<S>(label: S, on_pressed: SignalCallable) -> impl IntoControl
where
    S: Into<String> + ToGodot + Debug,
{
    control! { <button on:pressed=on_pressed text=label /> }
}

#[component]
fn Menu<S>(title: S) -> impl IntoControl
where
    S: Into<String> + ToGodot + Debug + Display,
{
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
            <hboxcontainer>
              <vboxcontainer>
                <For each=move || 0..count.get() key=|i| *i let(i)>
                    <label text=format!("Tick 1 {}", i) />
                </For>
              </vboxcontainer>
              <vboxcontainer>
                <For each=move || 0..count.get() key=|i| *i children=|i| {
                  control! { <label text=format!("Tick 2 {}", i) /> }
                } />
              </vboxcontainer>
              <vboxcontainer>
                <ForEnumerate each=move || 0..count.get() key=|i| *i let(idx, i)>
                    <label text=format!("Tick 3 {} ({})", i, idx.get()) />
                </ForEnumerate>
              </vboxcontainer>
              <vboxcontainer>
                <ForEnumerate each=move || 0..count.get() key=|i| *i children=|idx, i| {
                  control! { <label text=format!("Tick 4 {} ({})", i, idx.get()) /> }
                } />
              </vboxcontainer>
            </hboxcontainer>
            // event handling
            <button on:pressed=SignalCallable::new(move |_| {
                godot_print!("Button pressed! (count: {})", count.get());
                set_count.update(|c| *c += 1);
              })
              text=format!("Clicks: {}", count.get()) />
              // text=|| format!("Clicks: {}", count.get()) />
            // custom component usage
            <MenuButton label="Resume" on_pressed=resume />
            <MenuButton label="Quit" on_pressed=quit />
        </vboxcontainer>
    }
}

#[component]
fn Basic() -> impl IntoControl {
    let handler = SignalCallable::new(|_| {
        godot_print!("Resumed!");
    });
    control! {
        <panel />
        <vboxcontainer>
            <button on:pressed=handler text="Resume" />
            <button text="Save" />
            <button text="Load" />
            <Menu title="Pause" />
        </vboxcontainer>
    }
}

#[grui::prelude::class(root=Basic)]
pub struct PauseMenu {}

struct BasicExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BasicExtension {}
