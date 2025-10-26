use actix_web::{web, App, HttpServer, HttpResponse, Result};
use actix_files as afs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;
use regex::Regex;
use rusqlite::Connection;

#[derive(Serialize, Deserialize, Clone)]
struct Patient {
    cpf: String,
    nome: String,
    idade: String,
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
    let idade = extract_field(patient_section, "Idade:")?;

    let crm = extract_field(doctor_section, "CRM:")?;
    let doc_nome = extract_field(doctor_section, "Nome:")?;
    let especialidade = extract_field(doctor_section, "Especialidade:")?;

    let timestamp = extract_field(transaction_section, "Timestamp:")?;

    Some(Consultation {
        patient: Patient { cpf, nome, idade },
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
            idade TEXT
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

        conn.execute("INSERT OR REPLACE INTO patients (cpf, nome, idade) VALUES (?1, ?2, ?3)",
            [&cpf, &patient.nome, &patient.idade]).unwrap();

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
        patients_map.entry(key).or_insert_with(|| PatientSummary {
            patient: cons.patient.clone(),
            consultations: Vec::new(),
            current_medications: Vec::new(),
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
        patients_map.entry(key).or_insert_with(|| PatientSummary {
            patient: cons.patient.clone(),
            consultations: Vec::new(),
            current_medications: Vec::new(),
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
        patients_map.entry(key).or_insert_with(|| PatientSummary {
            patient: cons.patient.clone(),
            consultations: Vec::new(),
            current_medications: Vec::new(),
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
