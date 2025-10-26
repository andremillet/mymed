# Med Manager

Sistema modular para gerenciamento médico: criação, importação e listagem de pacientes, com subprogramas CLI e wrapper web.

**Versão Atual: 0.1.1**

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
- `mymed list patients`: Lista pacientes; interativo em terminal para selecionar e ver detalhes.
- `mymed upgrade`: Verifica e atualiza para a versão mais recente.

## Interface Web

- Botões/forms em index.html para "Novo Paciente", "Importar Paciente", "Listar Pacientes" (tabela interativa).

## Instalação

### Via Script (Recomendado)
Execute o comando abaixo no terminal para instalar automaticamente:

```bash
curl -fsSL https://raw.githubusercontent.com/andremillet/mymed/master/install.sh | bash
```

Isso baixa e instala o binário `mymed` em `/usr/local/bin`.

### Manual
1. Baixe o binário da [release mais recente](https://github.com/andremillet/mymed/releases).
2. `chmod +x mymed` e mova para `/usr/local/bin`.

### Desenvolvimento
1. `cargo build --release`
2. CLI: `./target/release/mymed new patient`
3. Web: `./target/release/mymed` → http://127.0.0.1:8080