pub struct Signals {
    pub rendering_enabled: bool,
}

impl Signals {
    pub fn new() -> Self {
        Self {
            rendering_enabled: false,
        }
    }
}