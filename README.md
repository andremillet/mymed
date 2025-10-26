# Med Manager

Sistema modular para gerenciamento médico: criação, importação e listagem de pacientes, com subprogramas CLI e wrapper web.

## Visão Geral

O projeto evolui para gestão de pacientes e atendimentos via arquivos .med, com compatibilidade FHIR.

## Estrutura do Projeto

- `src/main.rs`: Wrapper CLI/web.
- `src/patient_mgr.rs`: Lógica de pacientes.
- `medfiles/`: Arquivos .med.
- DB SQLite: `patients` (cpf, nome, birth_date).

## Comandos CLI (via `mymed`)

- `mymed new patient`: Prompt interativo para cadastrar paciente (CPF, nome, data nascimento DD/MM/AAAA).
- `mymed import patient`: Prompt para caminho .med, exibe [PATIENT], confirma importação.
- `mymed list patients`: Lista nomes de pacientes cadastrados/importados.

## Interface Web

- Botões/forms em index.html para "Novo Paciente", "Importar Paciente", "Listar Pacientes" (tabela).

## Instalação

1. `cargo build --release`
2. CLI: `./target/release/mymed new patient`
3. Web: `./target/release/mymed` → http://127.0.0.1:8080