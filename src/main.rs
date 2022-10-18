use std::{
    io::Read,
    process::{Child, Stdio},
};

use eframe::egui;

fn main() {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| Box::new(Terminal::default())),
    );
}

#[derive(Default)]
struct Terminal {
    processes: Vec<Process>,
    latest_command: String,
}

struct Process {
    id: uuid::Uuid,
    command: String,
    status: ProcessStatus,
}

impl Process {
    fn new(command: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            command,
            status: ProcessStatus::Waiting,
        }
    }
}

enum ProcessStatus {
    FailedToSpawn,
    Waiting,
    Running(Child),
    Finished(String),
}

impl eframe::App for Terminal {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // spawn all processes

        // render
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rTerm");

            for mut process in &mut self.processes {
                match &mut process.status {
                    ProcessStatus::FailedToSpawn => {}
                    ProcessStatus::Waiting => {
                        let cmd_args: Vec<String> = process
                            .command
                            .split_whitespace()
                            .map(ToOwned::to_owned)
                            .collect();

                        match std::process::Command::new(&cmd_args[0])
                            .stdout(Stdio::piped())
                            .args(&cmd_args[1..])
                            .spawn()
                        {
                            Ok(child) => process.status = ProcessStatus::Running(child),
                            Err(e) => {
                                tracing::error!(?e, ?process.command, "failed to spawn command");
                                process.status = ProcessStatus::FailedToSpawn;
                            }
                        }
                    }
                    ProcessStatus::Running(ref mut child) => {
                        ui.horizontal(|ui| {
                            ui.label(format!("Process {} running", process.id));
                        });

                        // see if process has finished
                        let finished = matches!(child.try_wait(), Ok(Some(_)));
                        if finished {
                            let child_output = child.stdout.as_mut().unwrap();
                            let mut output_text = String::new();
                            child_output.read_to_string(&mut output_text).unwrap();
                            process.status = ProcessStatus::Finished(output_text);
                        }
                    }
                    ProcessStatus::Finished(ref output) => {
                        ui.horizontal(|ui| {
                            ui.label(output);
                        });
                    }
                }
            }

            ui.horizontal(|ui| {
                ui.label("Command to run:");
                let response = ui.text_edit_singleline(&mut self.latest_command);
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    let c: String = self.latest_command.drain(..).collect();
                    tracing::info!("running command: {c}");
                    self.processes.push(Process::new(c));
                }

                // focus the text widget
                response.request_focus();
            });
        });
    }
}
