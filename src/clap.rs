use clap::{arg, Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Gpu type between AMD and Nvidia
    #[arg(short, long)]
    gpu_type: GpuType,
}

#[derive(Clone, ValueEnum)]
pub enum GpuType {
    Amd,
    Nvidia,
}
