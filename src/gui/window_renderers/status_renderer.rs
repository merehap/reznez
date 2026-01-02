use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;

pub struct StatusRenderer;

impl StatusRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;
}

impl WindowRenderer for StatusRenderer {
    fn name(&self) -> String {
        "Status".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let Some(nes) = &world.nes else {
            return FlowControl::CONTINUE;
        };

        let clock = nes.memory().ppu_regs.clock();
        let ppu_regs = &nes.memory().ppu_regs;
        let mem = nes.memory();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Frame");
                    ui.label(format!("{:?}", clock.frame()));
                    ui.end_row();
                    /*
                    ui.label("Scanline");
                    ui.label(format!("{:?}", clock.scanline()));
                    ui.end_row();
                    ui.label("PPU Cycle");
                    ui.label(format!("{:?}", clock.cycle()));
                    ui.end_row();
                    ui.label("CPU Cycle");
                    ui.label(format!("{:?}", nes.cpu().cycle()));
                    ui.end_row();
                    */
                    ui.label("X Scroll");
                    ui.label(format!("{}", ppu_regs.x_scroll().to_u8()));
                    ui.end_row();
                    ui.label("Y Scroll");
                    ui.label(format!("{}", ppu_regs.y_scroll().to_u8()));
                    ui.end_row();
                    ui.label("NMI Enabled");
                    ui.label(format!("{}", ppu_regs.nmi_enabled()));
                    ui.end_row();
                    ui.label("Sprite Height");
                    ui.label(format!("{:?}", ppu_regs.sprite_height()));
                    ui.end_row();
                    ui.label("Base Name Table");
                    ui.label(format!("{:?}", ppu_regs.base_name_table_quadrant()));
                    ui.end_row();
                    ui.label("Active Name Table");
                    ui.label(format!("{:?}", nes.memory().ppu_regs.active_name_table_quadrant()));
                    ui.end_row();
                    ui.label("Background");
                    ui.label(format!(
                        "Enabled: {}, Pattern Table: {:?} side",
                        ppu_regs.background_enabled(),
                        ppu_regs.background_table_side(),
                    ));
                    ui.end_row();
                    ui.label("Sprites");
                    ui.label(format!(
                        "Enabled: {}, Pattern Table: {:?} side",
                        ppu_regs.sprites_enabled(),
                        ppu_regs.sprite_table_side(),
                    ));
                    ui.end_row();
                    ui.label("");
                    ui.label("");
                    ui.end_row();
                    ui.label("Mapper");
                    ui.label(format!("{:?}", nes.resolved_metadata().mapper_number));
                    ui.end_row();
                    ui.label("Name Table Mirroring");
                    ui.label(format!("{}", mem.name_table_mirroring()));
                    ui.end_row();
                    ui.label("PRG ROM banks");
                    ui.label(nes.memory().prg_rom_bank_string());
                    ui.end_row();
                    ui.label("CHR ROM banks");
                    ui.label(nes.memory().chr_rom_bank_string());
                });
        });

        FlowControl::CONTINUE
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}