use log::{LevelFilter, error, info, warn};
use std::{ops::Deref, path::PathBuf};

use eframe::{
    NativeOptions,
    egui::{self, DroppedFile},
};

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
    root_dir: Option<PathBuf>,
    search_query: String,
}

impl Default for ImageViewer {
    fn default() -> Self {
        Self {
            current_src: None,
            image_size: [0, 0],
            files: Vec::new(),
            index: 0,
            randomize: true,
            root_dir: None,
            search_query: String::new(),
        }
    }
}

impl eframe::App for ImageViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let open_directory_requested =
            ctx.input(|i| i.modifiers == egui::Modifiers::CTRL && i.key_pressed(egui::Key::O));
        let focus_search_requested =
            ctx.input(|i| i.modifiers == egui::Modifiers::CTRL && i.key_pressed(egui::Key::F));
        let randomize_requested =
            ctx.input(|i| i.modifiers == egui::Modifiers::CTRL && i.key_pressed(egui::Key::R));

        let open_directory = |viewer: &mut Self| {
            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                if let Err(err) = viewer.open_dir(&dir) {
                    error!("Failed to open directory: {}", err);
                }
            }
        };

        let search = |viewer: &mut Self| {
            if let Some(dir) = viewer.root_dir.clone() {
                if let Err(err) = viewer.open_dir(&dir) {
                    error!("Failed to search directory: {}", err);
                }
            }
        };

        if open_directory_requested {
            open_directory(self);
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open directory").clicked() {
                    open_directory(self);
                }
                ui.label(self.root_dir.as_deref().map_or_else(
                    || "No folder selected".to_owned(),
                    |path| path.display().to_string(),
                ));
            });

            ui.horizontal(|ui| {
                ui.label("File filter:");
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .id(egui::Id::new("search_query")),
                );
                let search_requested = ui
                    .add_enabled(self.root_dir.is_some(), egui::Button::new("Search"))
                    .clicked()
                    || (search_response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)));

                if search_requested {
                    search(self);
                }

                if ui.button("Reset").clicked() {
                    self.search_query.clear();
                    search(self);
                }
                if focus_search_requested {
                    search_response.request_focus();
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let prev_random = self.randomize;
                if randomize_requested {
                    self.randomize = !self.randomize;
                }
                ui.toggle_value(&mut self.randomize, "Randomize");

                if prev_random != self.randomize && !self.files.is_empty() {
                    if self.randomize {
                        use rand::seq::SliceRandom;
                        let mut rng = rand::rng();
                        self.files.shuffle(&mut rng);
                    } else {
                        self.files.sort();
                    }
                    self.reindex();
                    // reset image_size so runtime loader can supply intrinsic size again
                    self.image_size = [0, 0];
                }

                if ui.button("First").clicked() {
                    self.first();
                }

                if ui.button("Prev").clicked() {
                    self.prev();
                }

                let position = if self.files.is_empty() {
                    "0 / 0".to_owned()
                } else {
                    format!("{} / {}", self.index + 1, self.files.len())
                };
                ui.label(position);

                if ui.button("Next").clicked() {
                    self.next();
                }

                if ui.button("Last").clicked() {
                    self.last();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let relative_path = self
                        .root_dir
                        .as_ref()
                        .and_then(|root| {
                            self.files
                                .get(self.index)
                                .and_then(|file| file.as_ref().strip_prefix(root).ok())
                        })
                        .map_or_else(|| "-".to_owned(), |path| path.display().to_string());
                    ui.label(relative_path);
                });
            });
        });

        // drag & drop: open directory or file
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() {
            self.open_dropped_path(dropped);
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
                        ui.add_sized(disp_size, egui::Image::new(src));
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
        self.root_dir = Some(dir.clone());
        let mut entries = Vec::new();
        collect_images(dir, &self.search_query, &mut entries).map_err(|e| e.to_string())?;

        if entries.is_empty() {
            self.files.clear();
            self.index = 0;
            self.current_src = None;
            self.image_size = [0, 0];
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
        self.image_size = [0, 0];

        info!(
            "Opened directory: {:?}, {} image files detected.",
            dir,
            self.files.len()
        );
        Ok(())
    }

    fn open_dropped_path(&mut self, dropped: Vec<DroppedFile>) {
        for f in dropped {
            let Some(dropped_path) = f.path.as_ref().map(|p| p.to_path_buf()) else {
                warn!("Dropped file has no path: {:?}", f.path);
                continue;
            };
            let dir_path = if dropped_path.is_dir() {
                dropped_path.clone()
            } else {
                let Some(parent) = dropped_path.parent() else {
                    warn!(
                        "Could not get parent directory of dropped file: {:?}",
                        dropped_path
                    );
                    continue;
                };
                parent.to_path_buf()
            };

            let _ = self.open_dir(&dir_path);
            info!(
                "Opened directory: {:?}, {} image files detected.",
                dir_path,
                self.files.len()
            );

            if dropped_path.is_file() {
                // display dropped file
                for (i, p) in self.files.iter().enumerate() {
                    if p.deref() == &dropped_path {
                        self.index = i;
                        self.current_src = Some(to_url(p.deref()));
                        break;
                    }
                }
                info!("Opened dropped file: {:?}", dropped_path);
                break;
            }
        }
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

    fn first(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.index = 0;
        let p = self.files[self.index].clone();
        self.current_src = Some(to_url(&p));
        self.image_size = [0, 0];
    }

    fn last(&mut self) {
        if self.files.is_empty() {
            return;
        }
        self.index = self.files.len() - 1;
        let p = self.files[self.index].clone();
        self.current_src = Some(to_url(&p));
        self.image_size = [0, 0];
    }

    fn reindex(&mut self) {
        let cur_path = self.current_src.as_ref().and_then(|s| to_path(s));
        if let Some(cur) = cur_path {
            let pos = self
                .files
                .iter()
                .position(|p| p.deref() == &cur)
                .unwrap_or(0);
            self.index = pos;
            self.current_src = Some(to_url(&self.files[self.index]));
        } else {
            self.index = 0;
            self.current_src = Some(to_url(&self.files[0]));
        }
    }
}

fn collect_images(
    dir: &PathBuf,
    search_query: &str,
    entries: &mut Vec<PathSortable>,
) -> std::io::Result<()> {
    let exts = [
        "jpg", "jpeg", "png", "bmp", "gif", "webp", "avif", "tif", "tiff",
    ];
    let query = search_query.to_ascii_lowercase();

    for entry in std::fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };

        if file_type.is_dir() && !search_query.is_empty() {
            collect_images(&path, search_query, entries)?;
            continue;
        }

        if file_type.is_dir() {
            continue;
        }

        if !file_type.is_file()
            || path
                .file_name()
                .is_none_or(|name| name.to_string_lossy().starts_with('.'))
        {
            continue;
        }

        let is_image = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| exts.contains(&extension.to_ascii_lowercase().as_str()))
            .unwrap_or(false);
        let matches_query = path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().to_ascii_lowercase().contains(&query));

        if is_image && matches_query {
            entries.push(PathSortable::from(path));
        }
    }

    Ok(())
}

fn main() {
    // initialize logger so egui_extras and other crates can emit diagnostics
    env_logger::builder()
        .default_format()
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
