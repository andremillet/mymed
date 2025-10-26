use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;
use crossterm::event::{read, Event, KeyCode, KeyEvent};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::io::{stdout, Write};

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
            let mut stmt = conn.prepare("SELECT nome, cpf, birth_date FROM patients ORDER BY nome").unwrap();
            let mut patients: Vec<(String, String, String)> = vec![];
            let patient_iter = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
            }).unwrap();
            for patient in patient_iter {
                patients.push(patient.unwrap());
            }
            if patients.is_empty() {
                println!("No patients found.");
                return;
            }
            enable_raw_mode().unwrap();
            let mut selected = 0;
            loop {
                // Clear screen and print list
                print!("\x1B[2J\x1B[1;1H");
                stdout().flush().unwrap();
                for (i, (nome, cpf, _)) in patients.iter().enumerate() {
                    if i == selected {
                        println!("> {}. {} - {}", i + 1, nome, cpf);
                    } else {
                        println!("  {}. {} - {}", i + 1, nome, cpf);
                    }
                }
                println!("\nUse ↑/↓ to navigate, Enter to select, 'q' to quit.");
                match read().unwrap() {
                    Event::Key(KeyEvent { code: KeyCode::Up, .. }) => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Down, .. }) => {
                        if selected < patients.len() - 1 {
                            selected += 1;
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                        let (nome, cpf, birth_date) = &patients[selected];
                        print!("\x1B[2J\x1B[1;1H");
                        stdout().flush().unwrap();
                        println!("Patient Details:");
                        println!("Name: {}", nome);
                        println!("CPF: {}", cpf);
                        println!("Birth Date: {}", birth_date);
                        println!("\nPress Enter to return to list...");
                        loop {
                            if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = read().unwrap() {
                                break;
                            }
                        }
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => {
                        break;
                    }
                    _ => {}
                }
            }
            disable_raw_mode().unwrap();
        }
    }
}