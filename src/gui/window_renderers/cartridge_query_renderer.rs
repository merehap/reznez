use egui::Context;
use egui_file::FileDialog;
use pixels::Pixels;

use crate::analysis::cartridge_db;
use crate::cartridge::resolved_metadata::{ResolvedMetadata, Vs};
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::mapper::KIBIBYTE;

pub struct CartridgeQueryRenderer {
    file_dialog: FileDialog,
}

impl CartridgeQueryRenderer {
    const WIDTH: usize = 950;
    const HEIGHT: usize = 400;

    pub fn new(file_dialog: FileDialog) -> Self {
        Self {
            file_dialog,
        }
    }
}

impl WindowRenderer for CartridgeQueryRenderer {
    fn name(&self) -> String {
        "ROM Query".to_string()
    }

    fn ui(&mut self, ctx: &Context, _world: &mut World) -> FlowControl {
        let result = FlowControl::CONTINUE;
        let mut should_close_file_dialog = false;
        egui::CentralPanel::default().show(ctx, |ui| {
            if !should_close_file_dialog {
                self.file_dialog.show(ctx);
            }

            if let Some(roms_directory) = self.file_dialog.path() && roms_directory.is_dir() {
                should_close_file_dialog = true;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("my_grid")
                        .num_columns(7)
                        .spacing([10.0, 2.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.label("Mapper");
                            ui.label("Submapper");
                            ui.label("Mirroring");
                            ui.label("Battery");
                            ui.label("Console");
                            ui.label("PRG ROM");
                            ui.label("PRG Work RAM");
                            ui.label("PRG Save RAM");
                            ui.label("CHR ROM");
                            ui.label("CHR Work RAM");
                            ui.label("CHR Save RAM");
                            ui.label("Region");
                            ui.label("Misc ROMs");
                            ui.label("Controller");
                            ui.label("VS Hardware");
                            ui.label("VS PPU");
                            ui.label("Full Hash");
                            ui.label("PRG ROM Hash");
                            ui.label("CHR ROM Hash");
                            ui.end_row();

                            for (path, metadata) in cartridge_db::analyze(roms_directory) {
                                let ResolvedMetadata { mapper_number, submapper_number, name_table_mirroring, has_persistent_memory,
                                    full_hash, prg_rom_hash, chr_rom_hash, prg_rom_size, prg_work_ram_size, prg_save_ram_size,
                                    chr_rom_size, chr_work_ram_size, chr_save_ram_size, console_type, region_timing_mode, miscellaneous_rom_count,
                                    default_expansion_device, vs } = metadata;
                                let mut vs_hardware_type = None;
                                let mut vs_ppu_type = None;
                                if let Some(Vs { hardware_type, ppu_type}) = vs {
                                    vs_hardware_type = Some(hardware_type);
                                    vs_ppu_type = Some(ppu_type);
                                }

                                fn kib_string(value: u32) -> String {
                                    if value < KIBIBYTE {
                                        value.to_string()
                                    } else {
                                        assert_eq!(value % KIBIBYTE, 0);
                                        format!("{}KiB", value / KIBIBYTE)
                                    }
                                }

                                ui.label(path.file_stem().map(|stem| stem.to_string_lossy().into_owned()).unwrap_or("???".to_owned()));
                                ui.label(mapper_number.to_string());
                                ui.label(submapper_number.map(|s| s.to_string()).unwrap_or("".to_owned()));
                                ui.label(name_table_mirroring.unwrap().to_string());
                                ui.label(has_persistent_memory.to_string());
                                ui.label(console_type.to_string());
                                ui.label(kib_string(prg_rom_size));
                                ui.label(kib_string(prg_work_ram_size));
                                ui.label(kib_string(prg_save_ram_size));
                                ui.label(kib_string(chr_rom_size));
                                ui.label(kib_string(chr_work_ram_size));
                                ui.label(kib_string(chr_save_ram_size));
                                ui.label(format!("{region_timing_mode:?}"));
                                ui.label(miscellaneous_rom_count.to_string());
                                ui.label(format!("{default_expansion_device:?}"));
                                ui.label(vs_hardware_type.map_or("".to_owned(), |hardware| format!("{hardware:?}")));
                                ui.label(vs_ppu_type.map_or("".to_owned(), |ppu| format!("{ppu:?}")));
                                ui.label(format!("{full_hash:X}"));
                                ui.label(format!("{prg_rom_hash:X}"));
                                ui.label(format!("{chr_rom_hash:X}"));
                                ui.end_row();
                            }
                        });
                });
            }
        });

        result
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