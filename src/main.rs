use crate::epi::Frame;
use crate::epi::Storage;
use eframe::{egui, epi};
use std::convert::TryInto;
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};

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
                        self.pid = *pid;
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
                    let process = match remoteprocess::Process::new(self.pid.try_into().unwrap()) {
                        Ok(x) => x,
                        Err(e) => {
                            self.message = e.to_string();
                            panic!("{}", e)
                        }
                    };
                    let _lock = process.lock();

                    // Create a stack unwind object, and use it to get the stack for each thread
                    let unwinder = process.unwinder().unwrap();
                    let symbolicator = process.symbolicator().unwrap();
                    for thread in process.threads().unwrap().iter() {
                        ui.label(format!(
                            "Thread {} - {}",
                            thread.id().unwrap(),
                            if thread.active().unwrap() {
                                "Running"
                            } else {
                                "Idle"
                            }
                        ));
                        ui.indent(format!("thread_{:?}_info", self.pid), |ui| {
                            let _lock = thread.lock().unwrap();
                            for ip in unwinder.cursor(&thread).unwrap() {
                                let ip = ip.unwrap();
                                symbolicator
                                    .symbolicate(ip, true, &mut |sf| {
                                        ui.label(format!("{}", sf));
                                    })
                                    .unwrap();
                            }
                        });
                    }
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
