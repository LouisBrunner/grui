use godot::prelude::*;
use godot_grui::prelude::*;

#[component]
fn AdvancedComp(root: Gd<Advanced>, some_data: ReadSignal<usize>) -> impl IntoControl {
    let clicked = SignalCallable::new(move |_| {
        root.signals().was_clicked().emit();
    });
    control! {
      <vboxcontainer anchor_right=1.0 anchor_bottom=1.0>
        <label text={some_data.get().to_string()} />
        <button on:pressed=clicked text="Signal" />
      </vboxcontainer>
    }
}

#[::godot_grui::prelude::class(root=AdvancedComp,forward=root)]
struct Advanced {
    #[prop(signal = "read")]
    some_data: usize,
}

#[godot_api]
impl Advanced {
    #[func]
    fn on_event(&mut self) {
        self.some_data.1.update(|prev| *prev += 1);
    }

    #[signal]
    fn was_clicked();
}
