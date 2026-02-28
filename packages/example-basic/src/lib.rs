use godot::{classes::control::SizeFlags, global::HorizontalAlignment, prelude::*};
use grui::prelude::*;
use std::fmt::Debug;

#[component]
fn MenuButton(#[prop(into)] label: String, on_pressed: SignalCallable) -> impl IntoControl {
    control! { <button on:pressed=on_pressed text=label.clone() /> }
}

#[component]
fn Menu(#[prop(into)] title: String) -> impl IntoControl {
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
                  let title = title.clone();
                  control! { <label text=format!("{} {}", title, i) /> }
              }).collect::<Vec<_>>()
            }
            // dynamic iteration
            <hboxcontainer anchor_left=0.5 anchor_right=0.5>
              <vboxcontainer size_flags_horizontal=SizeFlags::EXPAND_FILL>
                <For each=move || 0..count.get() key=|i| *i let(i)>
                  <label text=format!("Tick 1 {}", i) horizontal_alignment=if i % 2 == 0 { HorizontalAlignment::RIGHT } else { HorizontalAlignment::LEFT } />
                </For>
              </vboxcontainer>
              <vboxcontainer size_flags_horizontal=SizeFlags::EXPAND_FILL>
                <For each=move || 0..count.get() key=|i| *i children=|i| {
                  control! { <label text=format!("Tick 2 {}", i) horizontal_alignment=if i % 2 == 0 { HorizontalAlignment::RIGHT } else { HorizontalAlignment::LEFT } /> }
                } />
              </vboxcontainer>
              <vboxcontainer size_flags_horizontal=SizeFlags::EXPAND_FILL>
                <ForEnumerate each=move || 0..count.get() key=|i| *i let(idx, i)>
                    <label text=format!("Tick 3 {} ({})", i, idx.get()) horizontal_alignment=if i % 2 == 0 { HorizontalAlignment::RIGHT } else { HorizontalAlignment::LEFT } />
                </ForEnumerate>
              </vboxcontainer>
              <vboxcontainer size_flags_horizontal=SizeFlags::EXPAND_FILL>
                <ForEnumerate each=move || 0..count.get() key=|i| *i children=|idx, i| {
                  control! { <label text=format!("Tick 4 {} ({})", i, idx.get()) horizontal_alignment=if i % 2 == 0 { HorizontalAlignment::RIGHT } else { HorizontalAlignment::LEFT } /> }
                } />
              </vboxcontainer>
            </hboxcontainer>
            // event handling
            <button on:pressed=SignalCallable::new(move |_| {
                godot_print!("Button pressed! (count: {})", count.get());
                set_count.update(|c| *c += 1);
              })
              text=move || format!("Clicks: {}", count.get()) />
            // conditions
            {move || if count.get() > 3 {
              control!{ <label text="[SLOW1] STOP!" /> }.into_any()
            } else {
              control!{ <button text="[SLOW1] Keep pressing!" /> }.into_any()
            }}
            {move || if count.get() > 3 {
              control!{ <label text="[SLOW2] STOP!" /> }.into_any()
            } else {
              control!{ <label theme_override_font_sizes:font_size=30 text="[SLOW2] Keep pressing!" /> }.into_any()
            }}
            <Show
              when=move || {count.get() > 3}
              fallback=|| control!{ <button text="[FAST1] Keep pressing!" /> }
            >
                <label text="[FAST1] STOP!" />
            </Show>
            <Show
              when=move || {count.get() > 3}
            >
                <label text="[FAST2] STOP!" />
            </Show>
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
        <vboxcontainer anchor_right=1.0 anchor_bottom=1.0>
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
unsafe impl ExtensionLibrary for BasicExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Scene {
            LOGGER.install();
        }
    }
}

static LOGGER: grui::internal::logger::GodotLogger = grui::internal::logger::GodotLogger {};
