use log::{LevelFilter, error, info};
use std::{ops::Deref, path::PathBuf};

use eframe::{NativeOptions, egui};

mod path;
use crate::path::{PathSortable, to_path, to_url};

struct ImageViewer {
    // current image source as a URL or file:// URI that egui_extras can handle
    current_src: Option<String>,
    image_size: [usize; 2],
    // nothing diagnostic here — rely on runtime loaders
    files: Vec<PathSortable>,
    index: usize,
    randomize: bool,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            current_src: None,
            image_size: [0, 0],
            files: Vec::new(),
            index: 0,
            randomize: true,
        }
    }
}

impl eframe::App for ImageViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open directory").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        if let Err(err) = self.open_dir(&dir) {
                            error!("Failed to open directory: {}", err);
                        }
                    }
                }

                if ui.button("Prev").clicked() {
                    self.prev();
                }

                if ui.button("Next").clicked() {
                    self.next();
                }

                // detect toggle change so we can reorder files while keeping the current file visible
                let prev_random = self.randomize;
                ui.toggle_value(&mut self.randomize, "Randomize");

                if prev_random != self.randomize {
                    if !self.files.is_empty() {
                        // determine currently shown path (if any) without the file:// prefix
                        let cur_path = self
                            .current_src
                            .as_ref()
                            .and_then(|s| to_path(s));

                        if self.randomize {
                            use rand::seq::SliceRandom;
                            let mut rng = rand::rng();
                            self.files.shuffle(&mut rng);
                        } else {
                            self.files.sort();
                        }

                        // re-index to current file if present, otherwise fallback to 0
                        if let Some(cur) = cur_path {
                            if let Some(pos) = self.files.iter().position(|p| p.deref() == &cur) {
                                self.index = pos;
                                self.current_src = Some(to_url(&self.files[self.index]));
                            } else {
                                self.index = 0;
                                self.current_src = Some(to_url(&self.files[0]));
                            }
                        } else {
                            self.index = 0;
                            self.current_src = Some(to_url(&self.files[0]));
                        }
                        // reset image_size so runtime loader can supply intrinsic size again
                        self.image_size = [0, 0];
                    }
                }

                if let Some(src) = &self.current_src {
                    ui.label(src);
                }
            });
        });

        // drag & drop: open directory or file
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            for f in dropped {
                if let Some(p) = f.path {
                    if p.is_dir() {
                        let _ = self.open_dir(&p);
                        break;
                    } else if p.is_file() {
                        // open single file
                        self.files = vec![PathSortable::from(p.clone())];
                        self.index = 0;
                        self.current_src = Some(format!("file://{}", p.display()));
                        break;
                    }
                }
            }
        }

        // keyboard navigation
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.next();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.prev();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(src) = &self.current_src {
                // determine display size
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let avail = ui.available_size();
                        let disp_size = egui::vec2(avail.x, avail.y);
                        // Use egui Image widget with runtime source (egui_extras provides loaders)
                        ui.add_sized(disp_size, egui::Image::new(src.as_str()));
                    });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No image loaded. Click Open to choose an image.");
                });
            }
        });
    }
}

impl ImageViewer {
    fn open_dir(&mut self, dir: &PathBuf) -> Result<(), String> {
        let mut entries = std::fs::read_dir(dir)
            .map_err(|e| e.to_string())?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .is_some_and(|n| !n.to_string_lossy().starts_with('.'))
            })
            .map(PathSortable::from)
            .collect::<Vec<_>>();

        let exts = ["jpg", "jpeg", "png", "bmp", "gif", "webp", "avif"];
        entries.retain(|p| {
            p.extension()
                .and_then(|s| s.to_str())
                .map(|s| exts.contains(&s.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        });

        if entries.is_empty() {
            return Err("No image files found in directory".into());
        }

        if self.randomize {
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            entries.shuffle(&mut rng);
        } else {
            entries.sort();
        }

        self.files = entries;
        self.index = 0;
        let p = self.files[0].clone();

        self.current_src = Some(to_url(&p));
        // we don't know image size here; egui_extras may set it when loading. Keep fallback size 0.
        self.image_size = [0, 0];
        Ok(())
    }

    fn next(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.files.len();
        let p = self.files[self.index].clone();
        self.current_src = Some(to_url(&p));
        self.image_size = [0, 0];
    }

    fn prev(&mut self) {
        if self.files.is_empty() {
            return;
        }
        if self.index == 0 {
            self.index = self.files.len() - 1;
        } else {
            self.index -= 1;
        }
        let p = self.files[self.index].clone();
        self.current_src = Some(to_url(&p));
        self.image_size = [0, 0];
    }
}

fn main() {
    // initialize logger so egui_extras and other crates can emit diagnostics
    env_logger::builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .init();

    let options = NativeOptions::default();
    let _ = eframe::run_native(
        "view-rs",
        options,
        Box::new(|cc| {
            // install egui_extras image loaders so runtime image sources (file://, http://) work
            egui_extras::install_image_loaders(&cc.egui_ctx);
            info!("egui_extras image loaders installed");
            Ok(Box::new(ImageViewer::default()))
        }),
    );
}
