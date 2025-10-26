use actix_web::{web, App, HttpServer, HttpResponse, Result};
use actix_files as afs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;
use rusqlite::Connection;
use webbrowser;
use clap::{Parser, Subcommand};
use std::process::Command;
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
struct Patient {
    cpf: String,
    nome: String,
    birth_date: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Doctor {
    crm: String,
    nome: String,
    especialidade: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Consultation {
    patient: Patient,
    doctor: Doctor,
    timestamp: String,
    filename: String,
    hipotese_diagnostica: String,
    conduta: String,
}

#[derive(Serialize)]
struct Medication {
    name: String,
    dosage: String,
    start_date: String,
}

#[derive(Serialize)]
struct PatientSummary {
    patient: Patient,
    consultations: Vec<Consultation>,
    current_medications: Vec<Medication>,
    age: String,
}

#[derive(Parser)]
#[command(name = "mymed")]
#[command(about = "Medical management tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(name = "new")]
    New {
        #[command(subcommand)]
        sub: NewSub,
    },
    #[command(name = "import")]
    Import {
        #[command(subcommand)]
        sub: ImportSub,
    },
    #[command(name = "list")]
    List {
        #[command(subcommand)]
        sub: ListSub,
    },
    #[command(name = "web")]
    Web,
    #[command(name = "upgrade")]
    Upgrade,
}

#[derive(Subcommand)]
enum NewSub {
    #[command(name = "patient")]
    Patient,
}

#[derive(Subcommand)]
enum ImportSub {
    #[command(name = "patient")]
    Patient,
}

#[derive(Subcommand)]
enum ListSub {
    #[command(name = "patients")]
    Patients,
}

fn parse_med_file(content: &str) -> Option<Consultation> {
    let patient_re = Regex::new(r"(?s)\[PATIENT\]\s*(.*?)\[DOCTOR\]").unwrap();
    let doctor_re = Regex::new(r"(?s)\[DOCTOR\]\s*(.*?)\[TRANSACTION\]").unwrap();
    let transaction_re = Regex::new(r"(?s)\[TRANSACTION\]\s*(.*?)\[CONTRACT_CONDITIONS\]").unwrap();
    let diagnostica_re = Regex::new(r"(?s)\[HIPOTESE DIAGNOSTICA\]\s*(.*?)(?:\n\n|\[)").unwrap();
    let conduta_re = Regex::new(r"(?s)\[CONDUTA\]\s*(.*?)(?:\n\n|\[|$)").unwrap();

    let patient_section = patient_re.captures(content)?.get(1)?.as_str();
    let doctor_section = doctor_re.captures(content)?.get(1)?.as_str();
    let transaction_section = transaction_re.captures(content)?.get(1)?.as_str();
    let diagnostica = diagnostica_re.captures(content).map_or("", |c| c.get(1).map_or("", |m| m.as_str().trim()));
    let conduta = conduta_re.captures(content).map_or("", |c| c.get(1).map_or("", |m| m.as_str().trim()));

    let cpf = extract_field(patient_section, "CPF:")?;
    let nome = extract_field(patient_section, "Nome:")?;
    let idade_str = extract_field(patient_section, "Idade:")?;
    let idade_num: i32 = idade_str.parse().unwrap_or(0);
    let today = chrono::Utc::now().date_naive();
    let birth = today - chrono::Duration::days(idade_num as i64 * 365);
    let birth_date = birth.format("%Y-%m-%d").to_string();

    let crm = extract_field(doctor_section, "CRM:")?;
    let doc_nome = extract_field(doctor_section, "Nome:")?;
    let especialidade = extract_field(doctor_section, "Especialidade:")?;

    let timestamp = extract_field(transaction_section, "Timestamp:")?;

    Some(Consultation {
        patient: Patient { cpf, nome, birth_date },
        doctor: Doctor { crm, nome: doc_nome, especialidade },
        timestamp,
        filename: "".to_string(), // will set later
        hipotese_diagnostica: diagnostica.to_string(),
        conduta: conduta.to_string(),
    })
}

fn extract_field(section: &str, key: &str) -> Option<String> {
    for line in section.lines() {
        if line.starts_with(key) {
            return Some(line[key.len()..].trim().to_string());
        }
    }
    None
}

fn calculate_age(birth_date: &str) -> String {
    let birth = chrono::NaiveDate::parse_from_str(birth_date, "%Y-%m-%d").unwrap_or(chrono::Utc::now().date_naive());
    let today = chrono::Utc::now().date_naive();
    let age = (today - birth).num_days() / 365;
    age.to_string()
}

fn parse_patient_from_med(content: &str) -> Option<Patient> {
    let patient_re = Regex::new(r"(?s)\[PATIENT\]\s*(.*?)\[DOCTOR\]").unwrap();
    let patient_section = patient_re.captures(content)?.get(1)?.as_str();

    let cpf = extract_field(patient_section, "CPF:")?;
    let nome = extract_field(patient_section, "Nome:")?;
    let idade_str = extract_field(patient_section, "Idade:")?;
    let idade_num: i32 = idade_str.parse().ok()?;
    let today = chrono::Utc::now().date_naive();
    let birth = today - chrono::Duration::days(idade_num as i64 * 365);
    let birth_date = birth.format("%Y-%m-%d").to_string();

    Some(Patient { cpf, nome, birth_date })
}

fn load_consultations() -> Vec<Consultation> {
    let mut consultations = Vec::new();
    for entry in WalkDir::new("/home/woulschneider/petridish/mymed/medfiles").into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("med") {
            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                if let Some(mut cons) = parse_med_file(&content) {
                    if let Some(file_name) = entry.path().file_name().and_then(|n| n.to_str()) {
                        cons.filename = file_name.to_string();
                        consultations.push(cons);
                    }
                }
            }
        }
    }
    consultations
}

fn setup_db() {
    let conn = Connection::open("medications.db").unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS patients (
            cpf TEXT PRIMARY KEY,
            nome TEXT,
            birth_date TEXT
        )",
        [],
    ).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS medications (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE
        )",
        [],
    ).unwrap();
    conn.execute(
        "CREATE TABLE IF NOT EXISTS patient_medications (
            cpf TEXT,
            med_id INTEGER,
            status TEXT,
            dosage TEXT,
            start_date TEXT,
            end_date TEXT,
            FOREIGN KEY(cpf) REFERENCES patients(cpf),
            FOREIGN KEY(med_id) REFERENCES medications(id)
        )",
        [],
    ).unwrap();
}

fn process_medications(consultations: &[Consultation]) {
    let conn = Connection::open("medications.db").unwrap();
    let mut patients_map: HashMap<String, (Patient, Vec<Consultation>)> = HashMap::new();

    for cons in consultations {
        patients_map.entry(cons.patient.cpf.clone()).or_insert_with(|| (cons.patient.clone(), Vec::new())).1.push(cons.clone());
    }

    for (cpf, (patient, mut cons)) in patients_map {
        cons.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        let mut current_meds: HashMap<String, String> = HashMap::new(); // med -> dosage

            conn.execute("INSERT OR REPLACE INTO patients (cpf, nome, birth_date) VALUES (?1, ?2, ?3)",
             [&cpf, &patient.nome, &patient.birth_date]).unwrap();

        for c in &cons {
            let commands = parse_conduta_commands(&c.conduta);
            for (action, med, dosage) in commands {
                match action.as_str() {
                    "PRESCREVO" => {
                        current_meds.insert(med.clone(), dosage.clone());
                    }
                    "AJUSTO" => {
                        current_meds.insert(med.clone(), dosage.clone());
                    }
                    "SUSPENDO" => {
                        current_meds.remove(&med);
                    }
                    "MANTENHO" => {
                        // Keep if already present, or add if not (assuming previous prescription)
                        if !current_meds.contains_key(&med) {
                            current_meds.insert(med.clone(), dosage.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Clear old
        conn.execute("DELETE FROM patient_medications WHERE cpf = ?1", [&cpf]).unwrap();

        // Insert current
        for (med, dosage) in current_meds {
            conn.execute("INSERT OR IGNORE INTO medications (name) VALUES (?1)", [&med]).unwrap();
            let med_id = conn.query_row("SELECT id FROM medications WHERE name = ?1", [&med], |row| row.get::<_, i64>(0)).unwrap();
            conn.execute("INSERT INTO patient_medications (cpf, med_id, status, dosage, start_date) VALUES (?1, ?2, 'active', ?3, ?4)",
                (&cpf, &med_id, &dosage, &cons.first().unwrap().timestamp)).unwrap();
        }
    }
}

fn parse_conduta_commands(conduta: &str) -> Vec<(String, String, String)> {
    let mut commands = Vec::new();
    // Split by ; to get individual commands
    for part in conduta.split(';') {
        let part = part.trim();
        if part.starts_with('!') {
            let re_bracket = Regex::new(r"!(\w+)\s*\[([^\]]+)\](?:\s*\[([^\]]+)\])?").unwrap();
            let re_plain = Regex::new(r"!(\w+)\s*([^;\s]+)(?:\s*(.+))?").unwrap();
            let (action, med, dosage) = if let Some(cap) = re_bracket.captures(part) {
                (cap.get(1).unwrap().as_str().to_uppercase(),
                 cap.get(2).unwrap().as_str().to_string(),
                 cap.get(3).map_or("", |m| m.as_str()).to_string())
            } else if let Some(cap) = re_plain.captures(part) {
                (cap.get(1).unwrap().as_str().to_uppercase(),
                 cap.get(2).unwrap().as_str().to_string(),
                 cap.get(3).map_or("", |m| m.as_str()).to_string())
            } else {
                continue;
            };
            commands.push((action, med, dosage));
        }
    }
    commands
}

async fn get_patients(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse> {
    let consultations = load_consultations();
    let mut patients_map: HashMap<String, PatientSummary> = HashMap::new();

    for cons in consultations {
        let key = cons.patient.cpf.clone();
        patients_map.entry(key).or_insert_with(|| {
            let age = calculate_age(&cons.patient.birth_date);
            PatientSummary {
                patient: cons.patient.clone(),
                consultations: Vec::new(),
                current_medications: Vec::new(),
                age,
            }
        }).consultations.push(cons);
    }

    let mut patients: Vec<PatientSummary> = patients_map.into_iter().map(|(_, v)| v).collect();
    patients.sort_by(|a, b| a.patient.nome.cmp(&b.patient.nome));
    eprintln!("Grouped into {} patients", patients.len());

    let page: usize = query.get("page").and_then(|s| s.parse().ok()).unwrap_or(1);
    let limit: usize = query.get("limit").and_then(|s| s.parse().ok()).unwrap_or(10);
    let start = (page - 1) * limit;
    let _end = start + limit;
    let paginated = patients.into_iter().skip(start).take(limit).collect::<Vec<_>>();
    eprintln!("Returning {} patients for page {}", paginated.len(), page);

    Ok(HttpResponse::Ok().json(paginated))
}

async fn search_patients(query: web::Query<HashMap<String, String>>) -> Result<HttpResponse> {
    let consultations = load_consultations();
    let mut patients_map: HashMap<String, PatientSummary> = HashMap::new();

    for cons in consultations {
        let key = cons.patient.cpf.clone();
        patients_map.entry(key).or_insert_with(|| {
            let age = calculate_age(&cons.patient.birth_date);
            PatientSummary {
                patient: cons.patient.clone(),
                consultations: Vec::new(),
                current_medications: Vec::new(),
                age,
            }
        }).consultations.push(cons);
    }

    let patients: Vec<PatientSummary> = patients_map.into_iter().map(|(_, v)| v).collect();

    let filtered: Vec<PatientSummary> = if let Some(q) = query.get("q") {
        patients.into_iter().filter(|p|
            p.patient.nome.to_lowercase().contains(&q.to_lowercase()) ||
            p.patient.cpf.contains(q)
        ).collect()
    } else {
        patients
    };

    Ok(HttpResponse::Ok().json(filtered))
}

async fn get_patient(path: web::Path<String>) -> Result<HttpResponse> {
    let cpf = path.into_inner();
    let consultations = load_consultations();
    let mut patients_map: HashMap<String, PatientSummary> = HashMap::new();

    for cons in consultations {
        let key = cons.patient.cpf.clone();
        patients_map.entry(key).or_insert_with(|| {
            let age = calculate_age(&cons.patient.birth_date);
            PatientSummary {
                patient: cons.patient.clone(),
                consultations: Vec::new(),
                current_medications: Vec::new(),
                age,
            }
        }).consultations.push(cons);
    }

    if let Some(mut patient) = patients_map.remove(&cpf) {
        let conn = Connection::open("medications.db").unwrap();
        let mut stmt = conn.prepare("SELECT m.name, pm.dosage, pm.start_date FROM patient_medications pm JOIN medications m ON pm.med_id = m.id WHERE pm.cpf = ?1 AND pm.status = 'active'").unwrap();
        let meds_iter = stmt.query_map([&cpf], |row| {
            Ok(Medication {
                name: row.get(0)?,
                dosage: row.get(1)?,
                start_date: row.get(2)?,
            })
        }).unwrap();
        for med in meds_iter {
            patient.current_medications.push(med.unwrap());
        }
        Ok(HttpResponse::Ok().json(patient))
    } else {
        Ok(HttpResponse::NotFound().body("Patient not found"))
    }
}

async fn run_web() -> std::io::Result<()> {
    setup_db();
    let consultations = load_consultations();
    process_medications(&consultations);

    HttpServer::new(|| {
        App::new()
            .route("/patients", web::get().to(get_patients))
            .route("/search", web::get().to(search_patients))
            .route("/patient/{cpf}", web::get().to(get_patient))
            .service(afs::Files::new("/", ".").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        // run web
        run_web().await
    } else {
        // CLI
        let cli = Cli::parse();
        let result: std::io::Result<()> = match cli.command {
            Some(Commands::New { sub: NewSub::Patient }) => {
                println!("Digite CPF:");
                let mut cpf = String::new();
                std::io::stdin().read_line(&mut cpf).unwrap();
                let cpf = cpf.trim().to_string();
                println!("Digite nome:");
                let mut nome = String::new();
                std::io::stdin().read_line(&mut nome).unwrap();
                let nome = nome.trim().to_string();
                println!("Digite data nascimento (DD/MM/AAAA):");
                let mut birth_input = String::new();
                std::io::stdin().read_line(&mut birth_input).unwrap();
                let birth_input = birth_input.trim();
                // validate format
                if !regex::Regex::new(r"^\d{2}/\d{2}/\d{4}$").unwrap().is_match(birth_input) {
                    println!("Formato inválido. Use DD/MM/AAAA.");
                    return Ok(());
                }
                // convert to YYYY-MM-DD
                let parts: Vec<&str> = birth_input.split('/').collect();
                let birth_date = format!("{}-{}-{}", parts[2], parts[1], parts[0]);
                // call subprocess
                let status = Command::new("./target/debug/mymed_patient_manager")
                    .args(&["add", "--cpf", &cpf, "--nome", &nome, "--birth-date", &birth_date])
                    .status()
                    .expect("Failed to execute mymed_patient_manager");
                if status.success() {
                    println!("Paciente cadastrado com sucesso.");
                } else {
                    println!("Erro ao cadastrar paciente.");
                }
                Ok(())
            }
            Some(Commands::Import { sub: ImportSub::Patient }) => {
                println!("Digite caminho para arquivo .med:");
                let mut path = String::new();
                std::io::stdin().read_line(&mut path).unwrap();
                let path = path.trim();
                let content = match std::fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => {
                        println!("Arquivo não encontrado.");
                        return Ok(());
                    }
                };
                let patient = parse_patient_from_med(&content);
                if let Some(p) = patient {
                    println!("Dados do paciente:");
                    println!("CPF: {}", p.cpf);
                    println!("Nome: {}", p.nome);
                    println!("Data Nascimento: {}", p.birth_date);
                    println!("Confirmar importação? (s/n)");
                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm).unwrap();
                    if confirm.trim().to_lowercase() == "s" {
                        let status = Command::new("./target/debug/mymed_patient_manager")
                            .args(&["add", "--cpf", &p.cpf, "--nome", &p.nome, "--birth-date", &p.birth_date])
                            .status()
                            .expect("Failed");
                        if status.success() {
                            println!("Paciente importado.");
                        }
                    }
                } else {
                    println!("Falha ao parsear arquivo .med");
                }
                Ok(())
            }
            Some(Commands::List { sub: ListSub::Patients }) => {
                let output = Command::new("./target/debug/mymed_patient_manager")
                    .args(&["list"])
                    .output()
                    .expect("Failed");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                Ok(())
            }
            Some(Commands::Web) => {
                let _ = webbrowser::open("http://127.0.0.1:8080");
                run_web().await
            }
            Some(Commands::Upgrade) => {
                // check latest release
                let output = std::process::Command::new("gh")
                    .args(&["release", "list", "--json", "tagName", "--limit", "1"])
                    .output()
                    .expect("Failed to run gh");
                let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
                let latest_tag = json[0]["tagName"].as_str().unwrap();
                let current_version = env!("CARGO_PKG_VERSION");
                if latest_tag > current_version {
                    println!("Nova versão disponível: {}. Atualizando...", latest_tag);
                    // download
                    let download_output = std::process::Command::new("gh")
                        .args(&["release", "download", latest_tag, "--pattern", "mymed"])
                        .output()
                        .expect("Failed to download");
                    if download_output.status.success() {
                        // make executable and move
                        std::process::Command::new("chmod")
                            .args(&["+x", "mymed"])
                            .status()
                            .expect("Failed to chmod");
                        std::process::Command::new("sudo")
                            .args(&["mv", "mymed", "/usr/local/bin/mymed"])
                            .status()
                            .expect("Failed to move");
                        println!("Atualização concluída!");
                    } else {
                        println!("Erro ao baixar atualização.");
                    }
                } else {
                    println!("Você já tem a versão mais recente: {}", current_version);
                }
                Ok(())
            }
            None => {
                run_web().await
            }
        };
        result
    }
}
