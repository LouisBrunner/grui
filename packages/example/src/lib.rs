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

static LOGGER: grui::internal::logger::GodotLogger = grui::internal::logger::GodotLogger {};
