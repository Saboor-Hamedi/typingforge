//! CLI utility to generate `velotype_sentences.json` from classic public-domain literature.
//! Native Rust equivalent of `generateText.py`.

use forgetyping::utils::text_generator::TextGenerator;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let count: usize = if args.len() > 1 {
        args[1].parse().unwrap_or(5000)
    } else {
        5000
    };

    println!("[TextGenerator] Generating {count} clean, normalized literature practice passages...");
    let out_path = Path::new("velotype_sentences.json");

    match TextGenerator::generate_to_json_file(out_path, count) {
        Ok(n) => {
            println!("✓ Successfully generated {n} records to {}", out_path.display());
        }
        Err(e) => {
            eprintln!("✗ Error generating text: {e}");
            std::process::exit(1);
        }
    }
}
