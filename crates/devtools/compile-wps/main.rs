use clap::Parser;
use std::fs;
use std::path::PathBuf;
use watchpoint_parser::{parse_watchpoint_script, serialize_script};

#[derive(Parser)]
#[command(name = "wps")]
#[command(about = "Compiles watchpoint scripts to binary format", long_about = None)]
struct Cli {
    /// Input .wps script file
    input: PathBuf,
    
    /// Output .wpb binary file (optional, defaults to input with .wpb extension)
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();
    
    // Read the input file
    let source = match fs::read_to_string(&cli.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", cli.input.display(), e);
            std::process::exit(1);
        }
    };
    
    if cli.verbose {
        println!("Parsing {}...", cli.input.display());
    }
    
    // Parse the script
    let compiled = match parse_watchpoint_script(&source) {
        Ok(script) => script,
        Err(e) => {
            eprintln!("Compilation error: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("{:?}", compiled);
    
    if cli.verbose {
        println!("Compilation successful!");
        println!("  Variables: {}", compiled.variables.len());
        println!("  Conditions: {}", compiled.conditions.len());
        println!("  Init bytecode ops: {}", compiled.init_bytecode.len());
    }
    
    // Serialize the compiled script
    let binary = match serialize_script(&compiled) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Serialization error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Determine output path
    let output_path = cli.output.unwrap_or_else(|| {
        let mut path = cli.input.clone();
        path.set_extension("wpb");
        path
    });
    
    // Write the binary file
    if let Err(e) = fs::write(&output_path, &binary) {
        eprintln!("Error writing output file '{}': {}", output_path.display(), e);
        std::process::exit(1);
    }
    
    if cli.verbose {
        println!("Written {} bytes to {}", binary.len(), output_path.display());
    } else {
        println!("Compiled {} -> {}", cli.input.display(), output_path.display());
    }
}