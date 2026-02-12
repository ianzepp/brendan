use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

use brendan::layout;
use brendan::parse;
use brendan::render;

#[derive(Parser)]
#[command(name = "brendan", version, about = "Mermaid-compatible diagram renderer")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render a .mmd file to SVG
    Render {
        /// Input .mmd file
        input: PathBuf,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format
        #[arg(long, default_value = "svg")]
        format: String,
    },
    /// Extract ```mermaid blocks from markdown and render each
    Extract {
        /// Input markdown file
        input: PathBuf,
        /// Output directory for SVG files
        #[arg(long)]
        out_dir: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { input, output, format: _ } => {
            let content = fs::read_to_string(&input).unwrap_or_else(|e| {
                eprintln!("error reading {}: {}", input.display(), e);
                std::process::exit(1);
            });
            let svg = render_mermaid(&content);
            let out_path = output.unwrap_or_else(|| input.with_extension("svg"));
            fs::write(&out_path, &svg).unwrap_or_else(|e| {
                eprintln!("error writing {}: {}", out_path.display(), e);
                std::process::exit(1);
            });
            eprintln!("wrote {}", out_path.display());
        }
        Command::Extract { input, out_dir } => {
            let content = fs::read_to_string(&input).unwrap_or_else(|e| {
                eprintln!("error reading {}: {}", input.display(), e);
                std::process::exit(1);
            });
            fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
                eprintln!("error creating {}: {}", out_dir.display(), e);
                std::process::exit(1);
            });
            let blocks = extract_mermaid_blocks(&content);
            if blocks.is_empty() {
                eprintln!("no ```mermaid blocks found in {}", input.display());
                std::process::exit(1);
            }
            for (i, block) in blocks.iter().enumerate() {
                let svg = render_mermaid(block);
                let filename = format!("diagram-{}.svg", i + 1);
                let out_path = out_dir.join(&filename);
                fs::write(&out_path, &svg).unwrap_or_else(|e| {
                    eprintln!("error writing {}: {}", out_path.display(), e);
                    std::process::exit(1);
                });
                eprintln!("wrote {}", out_path.display());
            }
            eprintln!("{} diagrams extracted", blocks.len());
        }
    }
}

fn render_mermaid(input: &str) -> String {
    let diagram = parse::parse(input).unwrap_or_else(|e| {
        eprintln!("parse error: {}", e);
        std::process::exit(1);
    });
    match diagram {
        parse::Diagram::Sequence(seq) => {
            let canvas = layout::sequence::layout(&seq);
            render::svg::render(&canvas)
        }
    }
}

fn extract_mermaid_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = String::new();

    for line in content.lines() {
        if line.trim() == "```mermaid" {
            in_block = true;
            current.clear();
            continue;
        }
        if in_block && line.trim() == "```" {
            in_block = false;
            blocks.push(current.clone());
            continue;
        }
        if in_block {
            current.push_str(line);
            current.push('\n');
        }
    }
    blocks
}
