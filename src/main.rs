use core::time;
use std::collections::HashMap;
use std::thread;
use crate::epi::Frame;
use crate::epi::Storage;
use eframe::{egui, epi};
use sysinfo::{ProcessExt, System, SystemExt};
use proc_maps::{get_process_maps, MapRange, Pid};

fn wait(time: u64) {
    thread::sleep(time::Duration::from_secs(time));
}

fn organize(ui: &mut egui::Ui, data: Vec<MapRange>) {
    let mut organized: HashMap<String, HashMap<usize, usize>> = HashMap::new();
    for map in data {
        match map.filename() {
            Some(x) => {
                let name = &x.to_str();
                let _name = &*name.unwrap();
                if organized.contains_key(_name) {
                    let mut set = organized.get_mut(_name).unwrap();
                    set.insert(map.start(), map.size());
                } else {
                    organized.insert(_name.to_string(), HashMap::new());
                }
            },
            None => continue
        }
    }
    for (file, map) in organized {
        ui.collapsing(file, |ui| {
            for (address, size) in map {
                ui.label(format!("Starts at {} with size of {} ", address, size));
            }
        });

    }
}

#[derive(Default)]
struct SpecsGUI {
    sys: System,
    pid: usize,
    message: String,
}

impl epi::App for SpecsGUI {
    fn name(&self) -> &str {
        "MemReader"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &mut Frame<'_>,
        _storage: Option<&dyn Storage>,
    ) {
        self.pid = usize::MAX;
        self.sys = System::new_all();
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame<'_>) {
        self.sys.refresh_all();
        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let data = self.sys.processes();
                for (pid, process) in data {
                    let btn = ui.button(format!("{}/{}", pid, process.name()));
                    if btn.clicked() {
                        self.pid = *pid as usize;
                    }
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.pid == usize::MAX {
                ui.heading("Nothing here to see!");
            } else if self.message.len() != 0 {
                ui.heading(format!("{}", self.message));
            } else {
                ui.heading("Stack trace");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let maps = get_process_maps(self.pid as Pid).unwrap();
                    organize(ui, maps);
                })
            }
        });
    }
}

fn main() {
    let app = SpecsGUI::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
