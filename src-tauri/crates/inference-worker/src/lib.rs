mod artifact;
mod cancellation;
mod images;
mod initialization;
mod server;
pub use server::serve;
mod embedding;
mod onnx;
mod protocol;
mod speech;

pub fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return serve();
    }
    if args.len() != 4 || args[0] != "--audio-devices" {
        return Err("invalid_input: worker arguments are invalid".into());
    }
    let reserve = args[3]
        .parse::<u64>()
        .map_err(|_| "invalid_input: device reserve is invalid")?;
    if reserve > 32 * 1024 * 1024 * 1024 {
        return Err("invalid_input: device reserve exceeds its bound".into());
    }
    println!(
        "{}",
        speech::probe::devices(std::path::Path::new(&args[1]), &args[2], reserve)?
    );
    Ok(())
}
