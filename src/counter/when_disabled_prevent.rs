#[derive(Clone, Copy, Debug)]
pub enum WhenDisabledPrevent {
    Ticking,
    Triggering,
    TickingAndTriggering,
}