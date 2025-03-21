use std::sync::LazyLock;

use arr_macro::arr;

use crate::ppu::clock::Clock;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::scanline_actions::{
    ScanlineActions,
    FIRST_VISIBLE_SCANLINE_ACTIONS,
    VISIBLE_SCANLINE_ACTIONS,
    POST_RENDER_SCANLINE_ACTIONS,
    START_VBLANK_SCANLINE_ACTIONS,
    EMPTY_SCANLINE_ACTIONS,
    PRE_RENDER_SCANLINE_ACTIONS,
};

pub static NTSC_FRAME_ACTIONS: LazyLock<FrameActions> = LazyLock::new(|| {
    let mut ntsc_frame = FrameActions::new();

    ntsc_frame.set_scanline_actions_at(0, FIRST_VISIBLE_SCANLINE_ACTIONS.clone());
    for scanline in 1..=239 {
        ntsc_frame.set_scanline_actions_at(scanline, VISIBLE_SCANLINE_ACTIONS.clone());
    }

    // POST-RENDER SCANLINES
    ntsc_frame.set_scanline_actions_at(240, POST_RENDER_SCANLINE_ACTIONS.clone());
    ntsc_frame.set_scanline_actions_at(241, START_VBLANK_SCANLINE_ACTIONS.clone());
    for scanline in 242..=260 {
        ntsc_frame.set_scanline_actions_at(scanline, EMPTY_SCANLINE_ACTIONS.clone());
    }

    ntsc_frame.set_scanline_actions_at(261, PRE_RENDER_SCANLINE_ACTIONS.clone());

    ntsc_frame
});

#[derive(Clone)]
pub struct FrameActions {
    all_scanline_actions: Box<[ScanlineActions; 262]>,
}

impl FrameActions {
    pub fn current_cycle_actions(&self, clock: &Clock) -> &[CycleAction] {
        self.all_scanline_actions[clock.scanline() as usize].actions_at_cycle(clock.cycle())
    }

    pub fn format_current_cycle_actions(&self, clock: &Clock) -> String {
        match self.current_cycle_actions(clock) {
            [] => String::new(),
            [first, rest @ ..] => {
                let mut result = format!("{first:?}");
                for action in rest {
                    result.push_str(&format!(", {action:?}"));
                }

                result
            }
        }
    }

    fn new() -> FrameActions {
        FrameActions {
            all_scanline_actions: Box::new(arr![EMPTY_SCANLINE_ACTIONS.clone(); 262]),
        }
    }

    fn set_scanline_actions_at(&mut self, scanline: usize, scanline_actions: ScanlineActions) {
        self.all_scanline_actions[scanline] = scanline_actions;
    }
}
