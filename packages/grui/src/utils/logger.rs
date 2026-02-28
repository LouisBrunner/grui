use godot::global::{godot_error, godot_print, godot_warn};

pub struct GodotLogger {}

impl GodotLogger {
    pub fn install(&'static self) {
        log::set_max_level(log::STATIC_MAX_LEVEL);
        if let Err(err) = log::set_logger(self) {
            godot_error!("Failed to set logger: {}", err);
        }
    }
}

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
            log::Level::Info => godot_print!("INFO : {}", formatted),
            log::Level::Debug => godot_print!("DEBUG: {}", formatted),
            log::Level::Trace => godot_print!("TRACE: {}", formatted),
        }
    }

    fn flush(&self) {}
}
