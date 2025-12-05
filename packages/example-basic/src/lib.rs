use godot::prelude::*;
use grui::prelude::*;

#[component]
fn Basic() -> impl IntoControl {
    let handler = move |_: &[&Variant]| {
        godot_print!("Resumed!");
        Ok(Variant::nil())
    };
    control! {
        <panel />
        <vboxcontainer>
            <button on:click=handler text="Resume" />
            <button text="Save" />
            <button text="Load" />
        </vboxcontainer>
    }
}

#[grui::prelude::class(root=Basic)]
pub struct PauseMenu {}

struct BasicExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BasicExtension {}
