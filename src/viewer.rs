use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mymed_viewer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Patient(PatientArgs),
    File(FileArgs),
}

#[derive(Args)]
struct PatientArgs {
    #[arg(long)]
    cpf: String,
}

#[derive(Args)]
struct FileArgs {
    #[arg(long)]
    path: String,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Patient(args) => {
            // TODO: list consultations for cpf
            println!("View patient {}", args.cpf);
        }
        Commands::File(args) => {
            // TODO: parse and display file
            println!("View file {}", args.path);
        }
    }
}