#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    if let Err(error) = knovum_inference_worker::run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
