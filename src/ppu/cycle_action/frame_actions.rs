use arr_macro::arr;
use lazy_static::lazy_static;

use crate::ppu::clock::Clock;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::scanline_actions::{
    ScanlineActions,
    VISIBLE_SCANLINE_ACTIONS,
    EMPTY_SCANLINE_ACTIONS,
    START_VBLANK_SCANLINE_ACTIONS,
    VBLANK_SCANLINE_ACTIONS,
    PRE_RENDER_SCANLINE_ACTIONS,
};

lazy_static! {
    pub static ref NTSC_FRAME_ACTIONS: FrameActions = ntsc_frame_actions();
}

fn ntsc_frame_actions() -> FrameActions {
    let mut ntsc_frame = FrameActions::new();

    for scanline in 0..=239 {
        ntsc_frame.set_scanline_actions_at(scanline, VISIBLE_SCANLINE_ACTIONS.clone());
    }

    // POST-RENDER SCANLINES
    ntsc_frame.set_scanline_actions_at(         240, EMPTY_SCANLINE_ACTIONS.clone());
    ntsc_frame.set_scanline_actions_at(         241, START_VBLANK_SCANLINE_ACTIONS.clone());
    for scanline in 242..=260 {
        ntsc_frame.set_scanline_actions_at(scanline, VBLANK_SCANLINE_ACTIONS.clone());
    }

    ntsc_frame.set_scanline_actions_at(         261, PRE_RENDER_SCANLINE_ACTIONS.clone());

    ntsc_frame
}

#[derive(Clone)]
pub struct FrameActions {
    all_scanline_actions: Box<[ScanlineActions; 262]>,
}

impl FrameActions {
    pub fn current_cycle_actions(&self, clock: &Clock) -> &[CycleAction] {
        &self.all_scanline_actions[clock.scanline() as usize].actions_at_cycle(clock.cycle())
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