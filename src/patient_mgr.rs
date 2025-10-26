use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;

#[derive(Parser)]
#[command(name = "mymed_patient_manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    List,
}

#[derive(Args)]
struct AddArgs {
    #[arg(long)]
    cpf: String,
    #[arg(long)]
    nome: String,
    #[arg(long)]
    birth_date: String,
}

fn main() {
    let cli = Cli::parse();
    let conn = Connection::open("medications.db").unwrap();
    match cli.command {
        Commands::Add(args) => {
            // check if cpf exists
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM patients WHERE cpf = ?1", [&args.cpf], |row| row.get(0)).unwrap_or(0);
            if count > 0 {
                eprintln!("CPF já cadastrado.");
                std::process::exit(1);
            }
            conn.execute("INSERT INTO patients (cpf, nome, birth_date) VALUES (?1, ?2, ?3)",
                         [&args.cpf, &args.nome, &args.birth_date]).unwrap();
            println!("Paciente adicionado.");
        }
        Commands::List => {
            let mut stmt = conn.prepare("SELECT nome FROM patients ORDER BY nome").unwrap();
            let names = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            }).unwrap();
            for (i, name_result) in names.enumerate() {
                match name_result {
                    Ok(name) => println!("{}. {}", i+1, name),
                    Err(_) => {}
                }
            }
        }
    }
}