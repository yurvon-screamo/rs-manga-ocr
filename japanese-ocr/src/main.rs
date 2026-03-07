mod clipboard;
mod error;

use crate::clipboard::ClipboardHandler;
use crate::error::Error;
use clap::{Parser, ValueEnum};
use rs_japanese_ocr::JapaneseOCRModel;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    image: Option<PathBuf>,

    #[arg(long, default_value_t = Mode::Clipboard)]
    mode: Mode,

    #[arg(long, default_value = "1.0")]
    refresh_timeout: f64,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    File,
    Clipboard,
}
impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::File => write!(f, "file"),
            Mode::Clipboard => write!(f, "clipboard"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();
    let mut model = JapaneseOCRModel::load().await?;

    match args.mode {
        Mode::File => {
            if let Some(path) = args.image {
                let img = image::ImageReader::open(path)?.decode()?;
                let text = model.run(&img);

                match text {
                    Ok(text) => println!("{}", text),
                    Err(err) => println!("Error: {:?}", err),
                }
            } else {
                println!("No image provided");
            }
        }
        Mode::Clipboard => {
            let mut clipboard = ClipboardHandler::new(args.refresh_timeout)?;
            println!("Paste image (Ctrl+C to exit)");

            loop {
                tokio::select! {
                    biased;
                    _ = tokio::signal::ctrl_c() => {
                        println!("\nExiting...");
                        break;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                        let image = clipboard.get_image();
                        if let Some(img) = image {
                            let text = model.run(&img);
                            match text {
                                Ok(text) => {
                                    println!("{}", text);
                                    let _ = clipboard.set_text(&text);
                                }
                                Err(err) => println!("Error: {:?}", err),
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
