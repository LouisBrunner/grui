use godot::prelude::*;
use grui::prelude::*;
use std::fmt::{Debug, Display};

#[component]
fn MenuButton<S>(label: S, on_pressed: SignalCallback) -> impl IntoControl
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
    // let (count, set_count) = state(0);

    // Effect::new(|| {
    //     godot_print!("Effect: count is {}", count.get());
    // });

    let resume = SignalCallback::new(|_| {
        godot_print!("Resuming game!");
    });

    let quit = SignalCallback::new(|_| {
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
            // <For each=|| (0..count.get()) key=|i| *i let(i)>
            //     <label text=format!("Tick {}", i) />
            // </For>
            // event handling
            // <button on:pressed=Callable::from_fn(move || { set_count.update(|c| *c += 1); })
            //   text=format!("Clicks: {}", count.get()) />
            // custom component usage
            <MenuButton label="Resume" on_pressed=resume />
            <MenuButton label="Quit" on_pressed=quit />
        </vboxcontainer>
    }
}

#[component]
fn Basic() -> impl IntoControl {
    let handler = SignalCallback::new(|_| {
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
