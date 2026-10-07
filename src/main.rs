use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "clli",
    version = "1.0",
    author = "Fedi A. <fdiw@gmail.com>",
    about = "Trying things"
)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    /// Sets the input file to use
    input: String,

    /// Sets the level of verbosity
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Controls testing features
    Test {
        #[arg(short)]
        debug: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.config.as_deref() {
        Some(path) => println!("Config: {path}"),
        None => println!("No config file specified"),
    }

    println!("Input: {}", cli.input);
    println!("Verbosity: {}", cli.verbose);

    if let Some(Commands::Test { debug }) = cli.command {
        println!("Test debug: {debug}");
    }
}
