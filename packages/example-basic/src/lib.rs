use godot::{classes::control::SizeFlags, global::HorizontalAlignment, prelude::*};
use grui::prelude::*;
use std::fmt::Debug;

#[component]
fn MenuButton(label: String, on_pressed: SignalCallable) -> impl IntoControl {
    control! { <button on:pressed=on_pressed text=label.clone() /> }
}

#[component]
fn Menu(title: String) -> impl IntoControl {
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
              control!{ <label text="[SLOW] STOP!" /> }.into_any()
            } else {
              control!{ <button text="[SLOW] Keep pressing!" /> }.into_any()
            }}
            <Show
              when=move || {count.get() > 3}
              fallback=|| control!{ <button text="[FAST] Keep pressing!" /> }
            >
                <label text="[FAST] STOP!" />
            </Show>
            // custom component usage
            <MenuButton label="Resume".into() on_pressed=resume />
            <MenuButton label="Quit".into() on_pressed=quit />
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
            <Menu title="Pause".into() />
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
            log::set_max_level(log::STATIC_MAX_LEVEL);
            if let Err(err) = log::set_logger(&LOGGER) {
                godot_error!("Failed to set logger: {}", err);
            }
        }
    }
}

static LOGGER: GodotLogger = GodotLogger {};

pub struct GodotLogger {}

impl log::Log for GodotLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let formatted = format!(
            "[{}:{}] {}",
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        );
        match record.level() {
            log::Level::Error => godot_error!("{}", formatted),
            log::Level::Warn => godot_warn!("{}", formatted),
            log::Level::Info => godot_print!("{}", formatted),
            log::Level::Debug => godot_print!("DEBUG: {}", formatted),
            log::Level::Trace => godot_print!("TRACE: {}", formatted),
        }
    }

    fn flush(&self) {}
}
