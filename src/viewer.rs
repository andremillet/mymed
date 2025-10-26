use clap::{Args, Parser, Subcommand};
use rusqlite::Connection;
use chrono;

#[derive(Debug)]
struct Medication {
    name: String,
    dosage: String,
    start_date: String,
}

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

fn calculate_age(birth_date: &str) -> String {
    let birth = chrono::NaiveDate::parse_from_str(birth_date, "%Y-%m-%d").unwrap_or(chrono::Utc::now().date_naive());
    let today = chrono::Utc::now().date_naive();
    let age = (today - birth).num_days() / 365;
    age.to_string()
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Patient(args) => {
            let conn = Connection::open("medications.db").unwrap();
            // get patient
            let mut stmt = conn.prepare("SELECT nome, birth_date FROM patients WHERE cpf = ?1").unwrap();
            let patient = stmt.query_row([&args.cpf], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            });
            match patient {
                Ok((nome, birth_date)) => {
                    let age = calculate_age(&birth_date);
                    println!("Paciente: {} - CPF: {}", nome, args.cpf);
                    println!("Idade: {}", age);
                    // get medications
                    let mut med_stmt = conn.prepare("SELECT m.name, pm.dosage, pm.start_date FROM patient_medications pm JOIN medications m ON pm.med_id = m.id WHERE pm.cpf = ?1 AND pm.status = 'active'").unwrap();
                    let meds = med_stmt.query_map([&args.cpf], |row| {
                        Ok(Medication {
                            name: row.get(0)?,
                            dosage: row.get(1)?,
                            start_date: row.get(2)?,
                        })
                    }).unwrap();
                    println!("Medicações Atuais:");
                    for med in meds {
                        if let Ok(m) = med {
                            println!("- {} {} (desde {})", m.name, m.dosage, m.start_date);
                        }
                    }
                    // TODO: add consultations from files
                    println!("Consultas: (implementar parsing de .med)");
                }
                Err(_) => {
                    println!("Paciente não encontrado.");
                }
            }
        }
        Commands::File(args) => {
            // TODO: parse and display file
            println!("View file {}", args.path);
        }
    }
}