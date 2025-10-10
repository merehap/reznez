#[derive(Clone, Copy, Debug)]
pub enum WhenDisabledPrevent {
    Counting,
    Triggering,
    CountingAndTriggering,
}