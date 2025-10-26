# Med Viewer

Interface web para visualização e busca de arquivos médicos no formato .med.

## Funcionalidades

- Lista de pacientes com buscas por nome ou CPF.
- Visualização de histórico de consultas em linha do tempo.
- Backend em Rust com APIs REST.

## Estrutura do Arquivo .med

Os arquivos .med seguem o formato Medfile, com seções como [PATIENT], [DOCTOR], etc.

Exemplo de processamento:

`paciente_ficticio.med` >> `parse_med_file` >> `{"patient": {"cpf": "123.456.001-00", "nome": "Paciente Ficticio", "idade": "50"}, "doctor": {...}, "timestamp": "2025-10-12T10:00:00Z"}`

`arquivo .med` >> `extract_patient_info` >> `Patient { cpf: "123.456.001-00", nome: "Paciente Ficticio", idade: "50" }`

`consulta.med` >> `load_consultations` >> `[Consultation { patient: ..., doctor: ..., timestamp: ..., filename: "consulta.med" }]`

## APIs

- `GET /patients`: Lista todos os pacientes com suas consultas.
- `GET /search?q=termo`: Busca pacientes por nome ou CPF.

## Instalação e Execução

1. Instalar Rust: https://rustup.rs/
2. `cargo build --release`
3. `./target/release/med_viewer`
4. Abrir http://127.0.0.1:8080 no navegador.

## Dependências

- actix-web: Servidor web.
- regex: Parsing de seções.
- walkdir: Varredura de arquivos.
- serde: Serialização JSON.