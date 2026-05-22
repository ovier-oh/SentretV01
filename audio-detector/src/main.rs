use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;
use std::fs;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "audio-detector")]
#[command(version = "0.1.0")]
#[command(about = "Detector de audio en Rust")]

struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Setup,
}

#[derive(Serialize)]
struct AppConfig {
    app: AppSection,
    input: InputSection,
    buffer: BufferSection,
    processing: ProcessingSection,
    features: FeaturesSection,
    classifier: ClassifierSection,
    storage: StorageSection,
    ai: AiSection,
    output: OutputSection,
}

#[derive(Serialize)]
struct AppSection {
    name: String,
    version: String,
    log_level: String,
}

#[derive(Serialize)]
struct InputSection {
    device_name: String,
    sample_rate: u32,
    channels: u16,
    sample_format: String,
    backend: String,
}

#[derive(Serialize)]
struct BufferSection {
    buffer_size: u32,
    window_size: u32,
    hop_size: u32,
}

#[derive(Serialize)]
struct ProcessingSection {
    normalize: bool,
    highpass_hz: u32,
    lowpass_hz: u32,
}

#[derive(Serialize)]
struct FeaturesSection {
    enable_energy: bool,
    enable_zcr: bool,
    enable_fft: bool,
    enable_mfcc: bool,
    mfcc_coeffs: u8,
}

#[derive(Serialize)]
struct ClassifierSection {
    #[serde(rename = "type")]
    classifier_type: String,
    threshold: f32,
}

#[derive(Serialize)]
struct StorageSection {
    dataset_path: String,
    cache_features: bool,
}

#[derive(Serialize)]
struct AiSection {
    enabled: bool,
    model_path: String,
    input_type: String,
}

#[derive(Serialize)]
struct OutputSection {
    mode: String,
    events_file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup => setup()?,
    }
    Ok(())
}

fn setup() -> Result<()> {
    println!("Audio Detector Setup");
    println!("Search input devices");

    let host = cpal::default_host();

    let devices: Vec<_> = host
        .input_devices()
        .context("Input devices could not be read")?
        .collect();

    if devices.is_empty() {
        anyhow::bail!("No microphones were found connected");
    }

    for (index, device) in devices.iter().enumerate() {
        let name = device.name().unwrap_or_else(|_| "Unknown device".into());
        println!("[{}] {}", index, name);
    }

    print!("\nSelect device: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let selected_index: usize = input
        .trim()
        .parse()
        .context("You must enter a vaild number")?;

    let device = devices
        .get(selected_index)
        .context("The device index is out of range")?;

    let device_name = device.name().unwrap_or_else(|_| "Unknown device".into());

    let default_config = device
        .default_input_config()
        .context("Unable to retrieve microphone default settings")?;

    let sample_rate = default_config.sample_rate().0;
    let channels = default_config.channels();
    let sample_format = format!("{:?}", default_config.sample_format()).to_lowercase();

    println!("Select device: ");
    println!("Name: {}", device_name);
    println!("Sample rate: {} Hz", sample_rate);
    println!("Channels: {}", channels);
    println!("Format: {}", sample_format);

    let config = AppConfig {
        app: AppSection {
            name: "audio-detector".into(),
            version: "0.1.0".into(),
            log_level: "info".into(),
        },
        input: InputSection {
            device_name,
            sample_rate,
            channels,
            sample_format,
            backend: "auto".into(),
        },
        buffer: BufferSection {
            buffer_size: 1024,
            window_size: 2048,
            hop_size: 512,
        },
        processing: ProcessingSection {
            normalize: true,
            highpass_hz: 0,
            lowpass_hz: 0,
        },
        features: FeaturesSection {
            enable_energy: true,
            enable_zcr: true,
            enable_fft: false,
            enable_mfcc: false,
            mfcc_coeffs: 13,
        },
        classifier: ClassifierSection {
            classifier_type: "basic".into(),
            threshold: 0.7,
        },
        storage: StorageSection {
            dataset_path: "./data".into(),
            cache_features: true,
        },
        ai: AiSection {
            enabled: false,
            model_path: "./model.onnx".into(),
            input_type: "spectrogram".into(),
        },
        output: OutputSection {
            mode: "stdout".into(),
            events_file: "events.log".into(),
        },
    };

    let toml_text = toml::to_string_pretty(&config)?;
    fs::write("config.toml", toml_text)?;
    println!("\nGenerated file: config.toml");

    Ok(())
}
