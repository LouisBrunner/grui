use godot::prelude::*;

mod advanced;
mod basic;

struct BasicExtension;

#[gdextension]
unsafe impl ExtensionLibrary for BasicExtension {
    fn on_stage_init(stage: InitStage) {
        if stage == InitStage::Scene {
            LOGGER.install();
        }
    }
}

static LOGGER: godot_grui::internal::logger::GodotLogger = godot_grui::internal::logger::GodotLogger {};
