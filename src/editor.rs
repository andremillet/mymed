use clap::Parser;

#[derive(Parser)]
#[command(name = "mymed_editor")]
struct Cli {
    #[arg(long)]
    patient_cpf: String,
    #[arg(long)]
    doctor_crm: String,
}

fn main() {
    let cli = Cli::parse();
    // TODO: load patient, open editor for .med
    println!("Editor for patient {} by doctor {}", cli.patient_cpf, cli.doctor_crm);
}