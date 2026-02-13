use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

use crate::disk::DiskInfo;
use crate::flash::{self, FlashMessage};
use crate::image::ImageInfo;

const ACCENT: egui::Color32 = egui::Color32::from_rgb(60, 120, 220);
const ACCENT_HOVER: egui::Color32 = egui::Color32::from_rgb(80, 140, 240);
const DANGER: egui::Color32 = egui::Color32::from_rgb(220, 60, 60);
const SUCCESS: egui::Color32 = egui::Color32::from_rgb(60, 200, 100);
const MUTED: egui::Color32 = egui::Color32::from_rgb(140, 140, 150);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(35, 38, 45);
const CARD_SELECTED: egui::Color32 = egui::Color32::from_rgb(45, 55, 80);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Step {
    SelectImage,
    SelectDisk,
    Flash,
}

enum FlashState {
    Idle,
    Confirming,
    Flashing {
        progress: f32,
        bytes_written: u64,
        total_bytes: u64,
    },
    Done,
    Error(String),
}

pub struct InstallerApp {
    step: Step,
    images: Vec<ImageInfo>,
    selected_image: Option<usize>,
    target_dir: PathBuf,
    disks: Vec<DiskInfo>,
    selected_disk: Option<usize>,
    disk_error: Option<String>,
    flash_state: FlashState,
    flash_rx: Option<mpsc::Receiver<FlashMessage>>,
}

impl InstallerApp {
    pub fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let target_dir = if exe_dir.join("../target").exists() {
            exe_dir.join("../target")
        } else if PathBuf::from("../target").exists() {
            PathBuf::from("../target")
        } else {
            PathBuf::from("target")
        };

        let images = crate::image::scan_images(&target_dir);

        Self {
            step: Step::SelectImage,
            images,
            selected_image: None,
            target_dir,
            disks: Vec::new(),
            selected_disk: None,
            disk_error: None,
            flash_state: FlashState::Idle,
            flash_rx: None,
        }
    }

    fn poll_flash_progress(&mut self) {
        let mut finished = false;
        if let Some(rx) = &self.flash_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    FlashMessage::Progress {
                        bytes_written,
                        total_bytes,
                    } => {
                        let progress = if total_bytes > 0 {
                            bytes_written as f32 / total_bytes as f32
                        } else {
                            0.0
                        };
                        self.flash_state = FlashState::Flashing {
                            progress,
                            bytes_written,
                            total_bytes,
                        };
                    }
                    FlashMessage::Complete => {
                        self.flash_state = FlashState::Done;
                        finished = true;
                    }
                    FlashMessage::Error(e) => {
                        self.flash_state = FlashState::Error(e);
                        finished = true;
                    }
                }
            }
        }
        if finished {
            self.flash_rx = None;
        }
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.heading(egui::RichText::new("PiNAS Installer").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                let steps = ["Image", "Disk", "Flash"];
                let current = match self.step {
                    Step::SelectImage => 0,
                    Step::SelectDisk => 1,
                    Step::Flash => 2,
                };
                for (i, name) in steps.iter().enumerate().rev() {
                    let color = if i == current { ACCENT } else { MUTED };
                    let text = if i == current {
                        egui::RichText::new(*name).strong().color(color)
                    } else {
                        egui::RichText::new(*name).color(color)
                    };
                    ui.label(text);
                    if i > 0 {
                        ui.label(egui::RichText::new("/").color(MUTED).small());
                    }
                }
            });
        });
        ui.add_space(4.0);
    }

    fn render_card(
        ui: &mut egui::Ui,
        selected: bool,
        add_contents: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let bg = if selected { CARD_SELECTED } else { CARD_BG };
        let stroke = if selected {
            egui::Stroke::new(1.5, ACCENT)
        } else {
            egui::Stroke::NONE
        };
        let frame = egui::Frame::NONE
            .fill(bg)
            .stroke(stroke)
            .rounding(8)
            .inner_margin(egui::Margin::same(12));

        let response = frame
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                add_contents(ui);
            })
            .response;

        response.interact(egui::Sense::click())
    }

    fn render_step_image(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Select an image to flash")
                .size(17.0)
                .strong(),
        );
        ui.add_space(8.0);

        if self.images.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No .img.gz files found")
                        .size(16.0)
                        .color(MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!("Looking in: {}", self.target_dir.display()))
                        .small()
                        .color(MUTED),
                );
            });
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut clicked = None;
                for (i, image) in self.images.iter().enumerate() {
                    let selected = self.selected_image == Some(i);
                    let size = bytesize::ByteSize(image.size);
                    let date = crate::image::format_date(image.modified);

                    let resp = Self::render_card(ui, selected, |ui| {
                        ui.label(egui::RichText::new(&image.name).strong());
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{size}"))
                                    .small()
                                    .color(MUTED),
                            );
                            ui.label(
                                egui::RichText::new(format!("  {date}"))
                                    .small()
                                    .color(MUTED),
                            );
                        });
                    });

                    if resp.clicked() {
                        clicked = Some(i);
                    }
                    ui.add_space(4.0);
                }
                if let Some(i) = clicked {
                    self.selected_image = Some(i);
                }
            });
        }
    }

    fn render_step_disk(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Select target disk")
                .size(17.0)
                .strong(),
        );
        ui.add_space(8.0);

        if let Some(err) = &self.disk_error {
            ui.colored_label(DANGER, format!("{err}"));
            ui.add_space(4.0);
        }

        if self.disks.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new("No removable disks found")
                        .size(16.0)
                        .color(MUTED),
                );
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("Insert a USB drive or SD card and click Refresh")
                        .small()
                        .color(MUTED),
                );
            });
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut clicked = None;
                for (i, disk) in self.disks.iter().enumerate() {
                    let selected = self.selected_disk == Some(i);

                    let resp = Self::render_card(ui, selected, |ui| {
                        ui.label(
                            egui::RichText::new(if disk.name.is_empty() {
                                "USB Disk".to_string()
                            } else {
                                disk.name.clone()
                            })
                            .strong(),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("/dev/{}", disk.identifier))
                                    .small()
                                    .color(MUTED),
                            );
                            ui.label(
                                egui::RichText::new(format!("  {}", disk.size_display))
                                    .small()
                                    .color(MUTED),
                            );
                        });
                    });

                    if resp.clicked() {
                        clicked = Some(i);
                    }
                    ui.add_space(4.0);
                }
                if let Some(i) = clicked {
                    self.selected_disk = Some(i);
                }
            });
        }
    }

    fn render_step_flash(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let image = self.images[self.selected_image.unwrap()].clone();
        let disk = self.disks[self.selected_disk.unwrap()].clone();

        match &self.flash_state {
            FlashState::Idle => {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("Ready to flash").size(17.0).strong());
                ui.add_space(12.0);

                egui::Frame::NONE
                    .fill(CARD_BG)
                    .rounding(8)
                    .inner_margin(egui::Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::Grid::new("summary_grid")
                            .num_columns(2)
                            .spacing([16.0, 10.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Image").color(MUTED));
                                ui.label(egui::RichText::new(&image.name).strong());
                                ui.end_row();

                                ui.label(egui::RichText::new("Target").color(MUTED));
                                ui.label(egui::RichText::new(format!(
                                    "/dev/{} -- {} ({})",
                                    disk.identifier, disk.name, disk.size_display
                                )).strong());
                                ui.end_row();
                            });
                    });

                ui.add_space(16.0);

                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(60, 20, 20))
                    .rounding(8)
                    .inner_margin(egui::Margin::same(12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new("WARNING:")
                                    .color(DANGER)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(
                                    "The disk will be completely erased. All data will be lost.",
                                )
                                .color(egui::Color32::from_rgb(255, 180, 180)),
                            );
                        });
                    });
            }

            FlashState::Confirming => {
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Are you sure?")
                            .size(22.0)
                            .strong()
                            .color(DANGER),
                    );
                    ui.add_space(16.0);
                    ui.label(format!(
                        "This will erase /dev/{} ({}) and write:",
                        disk.identifier, disk.name
                    ));
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&image.name).strong().size(16.0));
                    ui.add_space(24.0);
                    ui.horizontal(|ui| {
                        let avail = ui.available_width();
                        ui.add_space((avail - 260.0) / 2.0);
                        if ui
                            .add_sized(
                                [120.0, 36.0],
                                egui::Button::new("Cancel"),
                            )
                            .clicked()
                        {
                            self.flash_state = FlashState::Idle;
                        }
                        ui.add_space(8.0);
                        if ui
                            .add_sized(
                                [120.0, 36.0],
                                egui::Button::new(
                                    egui::RichText::new("Confirm").color(egui::Color32::WHITE),
                                )
                                .fill(DANGER),
                            )
                            .clicked()
                        {
                            let (tx, rx) = mpsc::channel();
                            self.flash_rx = Some(rx);
                            self.flash_state = FlashState::Flashing {
                                progress: 0.0,
                                bytes_written: 0,
                                total_bytes: 0,
                            };
                            flash::start_flash(
                                &image.path,
                                &disk.identifier,
                                image.size,
                                tx,
                            );
                        }
                    });
                });
            }

            FlashState::Flashing {
                progress,
                bytes_written,
                total_bytes,
            } => {
                let progress = *progress;
                let bytes_written = *bytes_written;
                let total_bytes = *total_bytes;

                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Flashing...")
                            .size(20.0)
                            .strong(),
                    );
                    ui.add_space(20.0);

                    let bar = egui::ProgressBar::new(progress)
                        .text(format!(
                            "{} / {}  ({:.0}%)",
                            bytesize::ByteSize(bytes_written),
                            bytesize::ByteSize(total_bytes),
                            progress * 100.0
                        ))
                        .animate(true)
                        .fill(ACCENT);
                    ui.add_sized([ui.available_width() * 0.85, 28.0], bar);

                    ui.add_space(12.0);
                    ui.label(
                        egui::RichText::new("Do not remove the disk")
                            .color(egui::Color32::from_rgb(255, 200, 80)),
                    );
                });
                ctx.request_repaint();
            }

            FlashState::Done => {
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Flash complete!")
                            .size(24.0)
                            .strong()
                            .color(SUCCESS),
                    );
                    ui.add_space(16.0);
                    ui.label(
                        egui::RichText::new("The disk has been ejected safely.")
                            .size(15.0),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new("You can now insert it into your Raspberry Pi 5.")
                            .size(15.0)
                            .color(MUTED),
                    );
                });
            }

            FlashState::Error(err) => {
                let err_msg = err.clone();
                ui.add_space(60.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Flash failed")
                            .size(24.0)
                            .strong()
                            .color(DANGER),
                    );
                    ui.add_space(16.0);
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgb(60, 20, 20))
                        .rounding(8)
                        .inner_margin(egui::Margin::same(12))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(&err_msg)
                                    .color(egui::Color32::from_rgb(255, 180, 180)),
                            );
                        });
                    ui.add_space(16.0);
                    if ui
                        .add_sized([120.0, 36.0], egui::Button::new("Retry"))
                        .clicked()
                    {
                        self.flash_state = FlashState::Idle;
                    }
                });
            }
        }
    }

    fn render_footer(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(4.0);

            // Left side buttons
            match self.step {
                Step::SelectImage => {
                    if ui
                        .add_sized([100.0, 34.0], egui::Button::new("Refresh"))
                        .clicked()
                    {
                        self.images = crate::image::scan_images(&self.target_dir);
                        self.selected_image = None;
                    }
                }
                Step::SelectDisk => {
                    if ui
                        .add_sized([100.0, 34.0], egui::Button::new("Refresh"))
                        .clicked()
                    {
                        self.refresh_disks();
                    }
                }
                Step::Flash => {}
            }

            // Right side buttons
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                let is_busy = matches!(
                    self.flash_state,
                    FlashState::Flashing { .. } | FlashState::Confirming
                );

                match self.step {
                    Step::SelectImage => {
                        let enabled = self.selected_image.is_some();
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new(
                                    egui::RichText::new("Next >>")
                                        .color(if enabled {
                                            egui::Color32::WHITE
                                        } else {
                                            MUTED
                                        }),
                                )
                                .fill(if enabled { ACCENT } else { CARD_BG })
                                .min_size(egui::vec2(110.0, 34.0)),
                            )
                            .clicked()
                        {
                            self.step = Step::SelectDisk;
                            self.refresh_disks();
                        }
                    }
                    Step::SelectDisk => {
                        let enabled = self.selected_disk.is_some();
                        if ui
                            .add_enabled(
                                enabled,
                                egui::Button::new(
                                    egui::RichText::new("Next >>")
                                        .color(if enabled {
                                            egui::Color32::WHITE
                                        } else {
                                            MUTED
                                        }),
                                )
                                .fill(if enabled { ACCENT } else { CARD_BG })
                                .min_size(egui::vec2(110.0, 34.0)),
                            )
                            .clicked()
                        {
                            self.step = Step::Flash;
                            self.flash_state = FlashState::Idle;
                        }
                        if ui
                            .add_sized([100.0, 34.0], egui::Button::new("<< Back"))
                            .clicked()
                        {
                            self.step = Step::SelectImage;
                        }
                    }
                    Step::Flash if !is_busy => match &self.flash_state {
                        FlashState::Idle => {
                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("Flash!")
                                            .strong()
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(ACCENT)
                                    .min_size(egui::vec2(110.0, 34.0)),
                                )
                                .clicked()
                            {
                                self.flash_state = FlashState::Confirming;
                            }
                            if ui
                                .add_sized([100.0, 34.0], egui::Button::new("<< Back"))
                                .clicked()
                            {
                                self.step = Step::SelectDisk;
                            }
                        }
                        FlashState::Done | FlashState::Error(_) => {
                            if ui
                                .add_sized(
                                    [120.0, 34.0],
                                    egui::Button::new("Start Over"),
                                )
                                .clicked()
                            {
                                self.step = Step::SelectImage;
                                self.selected_image = None;
                                self.selected_disk = None;
                                self.flash_state = FlashState::Idle;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        });
        ui.add_space(4.0);
    }

    fn refresh_disks(&mut self) {
        match crate::disk::list_external_disks() {
            Ok(disks) => {
                self.disks = disks;
                self.disk_error = None;
            }
            Err(e) => {
                self.disk_error = Some(e);
            }
        }
        self.selected_disk = None;
    }
}

impl eframe::App for InstallerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_flash_progress();

        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(25, 28, 35))
                    .inner_margin(egui::Margin::same(4)),
            )
            .show(ctx, |ui| {
                self.render_header(ui);
            });

        egui::TopBottomPanel::bottom("footer")
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(25, 28, 35))
                    .inner_margin(egui::Margin::same(4)),
            )
            .show(ctx, |ui| {
                self.render_footer(ui);
            });

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(30, 33, 40))
                    .inner_margin(egui::Margin::symmetric(20, 8)),
            )
            .show(ctx, |ui| {
                match self.step {
                    Step::SelectImage => self.render_step_image(ui),
                    Step::SelectDisk => self.render_step_disk(ui),
                    Step::Flash => self.render_step_flash(ui, ctx),
                }
            });
    }
}
