use config::Config;
use hart::{StepState, Hart};
use machine::Machine;

mod machine;
mod debug;
mod hart;
mod gpu;
mod run;
mod debugger;
mod run_debugger;
mod ui;
mod hart_clock;
mod interrupt_controller;
mod spu;
mod config;

use run::run;
use run_debugger::run_debugger;
use ui::main_window;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let debug = args.iter().any(|x| x.contains("-d"));
    let config = Config::default();
    
    ui::main_window::MainWindow::run(&config, move |main_window| {
        let rom_bytes = include_bytes!("../test_binaries/cutout_blit/bin/cutout_blit.bin");
        let machine = Machine::new(&rom_bytes[..], main_window.clone());
        if debug {
            run_debugger(machine.clone(), None);
        } else {
            run(&machine);
        }
        main_window.exit();
    });
}
