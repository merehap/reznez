use egui::Context;
use pixels::Pixels;

use crate::cartridge::resolved_metadata::{ResolvedMetadata, Vs};
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::mapper::KIBIBYTE;

pub struct CartridgeMetadataRenderer;

impl CartridgeMetadataRenderer {
    const WIDTH: usize = 650;
    const HEIGHT: usize = 300;
}

impl WindowRenderer for CartridgeMetadataRenderer {
    fn name(&self) -> String {
        "Status".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let Some(nes) = &world.nes else {
            return FlowControl::CONTINUE;
        };

        let resolver = nes.metadata_resolver();
        // Explicitly spell out each field so that if a new one is added, a compile error occurs so it becomes apparent that the
        // field needs to be added to the GUI.
        let ResolvedMetadata { mapper_number, submapper_number, name_table_mirroring, has_persistent_memory,
            full_hash: _, prg_rom_hash: _, chr_rom_hash: _, prg_rom_size, prg_work_ram_size, prg_save_ram_size,
            chr_rom_size, chr_work_ram_size, chr_save_ram_size, console_type, region_timing_mode, miscellaneous_rom_count,
            default_expansion_device, vs } = resolver.resolve();
        let mut vs_hardware_type = None;
        let mut vs_ppu_type = None;
        if let Some(Vs { hardware_type, ppu_type}) = vs {
            vs_hardware_type = Some(hardware_type);
            vs_ppu_type = Some(ppu_type);
        }

        let metadata_sources = [
            &resolver.hard_coded_overrides,
            &resolver.cartridge,
            &resolver.database,
            &resolver.database_extension,
            &resolver.defaults(),
        ];

        egui::CentralPanel::default().show(ctx, |ui| {
            fn kib_string(value: u32) -> String {
                if value < KIBIBYTE {
                    value.to_string()
                } else {
                    assert_eq!(value % KIBIBYTE, 0);
                    format!("{}KiB", value / KIBIBYTE)
                }
            }

            egui::Grid::new("my_grid")
                .num_columns(7)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Field");
                    ui.label("ACTUAL");
                    ui.label("Hard-coded Overrides");
                    ui.label("Cartridge");
                    ui.label("Header Database");
                    ui.label("Database Extension");
                    ui.label("Defaults");
                    ui.end_row();

                    ui.label("Mapper");
                    ui.label(mapper_number.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.mapper_number().map_or(String::new(), |m| m.to_string()));
                    }
                    ui.end_row();

                    ui.label("Submapper");
                    ui.label(submapper_number.map_or("N/A".to_owned(), |s| s.to_string()));
                    for metadata in metadata_sources {
                        ui.label(metadata.submapper_number().map_or(String::new(), |m| m.to_string()));
                    }
                    ui.end_row();

                    ui.label("Name Table Mirroring");
                    ui.label(name_table_mirroring.unwrap().to_string());
                    for metadata in metadata_sources {
                        let mut text = String::new();
                        if let Some(mirroring) = metadata.name_table_mirroring() {
                            text.push_str(&mirroring.to_string());
                        }

                        if let Some(index) = metadata.name_table_mirroring_index() {
                            text.push_str(" (index = ");
                            text.push_str(&index.to_string());
                            text.push(')');
                        }

                        ui.label(text);
                    }
                    ui.end_row();

                    ui.label("Has Persistent Memory");
                    ui.label(has_persistent_memory.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.has_persistent_memory().map_or(String::new(), |m| m.to_string()));
                    }
                    ui.end_row();

                    ui.label("Console Type");
                    ui.label(console_type.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.console_type().map_or(String::new(), |m| m.to_string()));
                    }
                    ui.end_row();

                    ui.label("PRG ROM Size");
                    ui.label(kib_string(prg_rom_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_rom_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("PRG Work RAM Size");
                    ui.label(kib_string(prg_work_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_work_ram_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("PRG Save RAM Size");
                    ui.label(kib_string(prg_save_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_save_ram_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("CHR ROM Size");
                    ui.label(kib_string(chr_rom_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_rom_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("CHR Work RAM Size");
                    ui.label(kib_string(chr_work_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_work_ram_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("CHR Save RAM Size");
                    ui.label(kib_string(chr_save_ram_size));
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_save_ram_size().map_or(String::new(), kib_string));
                    }
                    ui.end_row();

                    ui.label("Region Timing Mode");
                    ui.label(format!("{region_timing_mode:?}"));
                    for metadata in metadata_sources {
                        ui.label(metadata.timing_mode().map_or(String::new(), |region| format!("{region:?}")));
                    }
                    ui.end_row();

                    ui.label("Miscellaneous ROM Count");
                    ui.label(miscellaneous_rom_count.to_string());
                    for metadata in metadata_sources {
                        ui.label(metadata.miscellaneous_rom_count().map_or(String::new(), |count| format!("{count:?}")));
                    }
                    ui.end_row();

                    ui.label("Default Expansion Device");
                    ui.label(format!("{default_expansion_device:?}"));
                    for metadata in metadata_sources {
                        ui.label(metadata.default_expansion_device().map_or(String::new(), |device| format!("{device:?}")));
                    }
                    ui.end_row();

                    ui.label("VS Hardware Type");
                    ui.label(vs_hardware_type.map_or(String::new(), |hardware| format!("{hardware:?}")));
                    for metadata in metadata_sources {
                        ui.label(metadata.vs_hardware_type().map_or(String::new(), |hardware| format!("{hardware:?}")));
                    }
                    ui.end_row();

                    ui.label("VS PPU Type");
                    ui.label(vs_ppu_type.map_or(String::new(), |ppu| format!("{ppu:?}")));
                    for metadata in metadata_sources {
                        ui.label(metadata.vs_ppu_type().map_or(String::new(), |ppu| format!("{ppu:?}")));
                    }
                    ui.end_row();

                    ui.label("Full CRC");
                    ui.label("");
                    for metadata in metadata_sources {
                        ui.label(metadata.full_hash().map_or(String::new(), |crc| format!("{crc:X}")));
                    }
                    ui.end_row();

                    ui.label("PRG ROM CRC");
                    ui.label("");
                    for metadata in metadata_sources {
                        ui.label(metadata.prg_rom_hash().map_or(String::new(), |crc| format!("{crc:X}")));
                    }
                    ui.end_row();

                    ui.label("CHR ROM CRC");
                    ui.label("");
                    for metadata in metadata_sources {
                        ui.label(metadata.chr_rom_hash().map_or(String::new(), |crc| format!("{crc:X}")));
                    }
                    ui.end_row();
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