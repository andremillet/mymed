# Medfile: O Formato de Documentação Médica Digital

## Descrição
O Medfile é um formato de smart contract para registros médicos digitais, baseado em arquivos `.med`. Esses arquivos atuam como contratos inteligentes médicos, estruturados em plain text legível com seções delimitadas por colchetes. Cada arquivo representa uma transação médica imutável, com paciente como alvo, médico como promotor, e condições executáveis.

As seções incluem:
- **[PATIENT]**: Dados do paciente (CPF hasheado, nome, idade).
- **[DOCTOR]**: Dados do médico (CRM, nome, especialidade).
- **[TRANSACTION]**: Metadados (ID, timestamp, hash, assinatura digital).
- **[CONTRACT_CONDITIONS]**: Termos executáveis (ex.: consentimento, certificação).
- **[CONTENT]**: Corpo do registro, com [CONDUTA] contendo comandos parseáveis para automação.

Isso permite execução condicional de intervenções, como liberar medicações apenas se condições forem atendidas.

### Aspectos do Formato Medfile
- **Estrutura Geral**: Arquivos .med são smart contracts médicos, começando com seções de metadados ([PATIENT], [DOCTOR], [TRANSACTION]), seguidos de [CONTENT] com o registro clínico. Dentro de [CONTENT], seções como [ANAMNESE], [EXAME FISICO], [CONDUTA] usam símbolos para ações:
  - `+`: Adições (ex.: medicações, exames).
  - `!`: Notas/histórico (ex.: antecedentes).
  - `-`: Remoções/alterações (ex.: trocas).
- **Exemplo de Smart Contract .med**:
  ```
  [PATIENT]
  CPF: 123.456.789-00
  Nome: João Silva
  Idade: 65

  [DOCTOR]
  CRM: 54321-RJ
  Nome: Dr. Ana Pereira
  Especialidade: Neurologia

  [TRANSACTION]
  ID: tx001
  Timestamp: 2025-10-12T14:00:00Z
  Hash: sha256_placeholder
  Signature: assinatura_digital

  [CONTRACT_CONDITIONS]
  - Paciente consente com o tratamento.
  - Médico certifica o diagnóstico.
  - Transação imutável após assinatura.

  [CONTENT]
  [ANAMNESE]
  PACIENTE COM DOR TORACICA RECORRENTE.
  !HPP HAS; !MED LOSARTANA 50MG;

  [EXAME FISICO]
  PRESSAO ARTERIAL 140/90; FREQUENCIA CARDIACA 80BPM.

  [CONDUTA]
  !PRESCREVO [DIPIRONA] [500MG] [6/6 HORAS];
  !ENCAMINHO [CARDIOLOGISTA];
  !SUSPENDO [PARACETAMOL] (SUBSTITUIDO POR DIPIRONA);
  ```
- **Execução Condicional**: Condições em [CONTRACT_CONDITIONS] permitem validação antes de executar ações (ex.: liberar medicação apenas com consentimento).
- **Flexibilidade e Padronização**: Suporta variações, mas sintaxe consistente. Privacidade via hashing de CPF.
- **Compatibilidade**: Integrável com sistemas EHR; parseável para JSON/Blockchain.

## Extração e Parsing de Condutas
A seção [CONDUTA] é o coração do Medfile, contendo comandos padronizados que podem ser parseados para automação. Usa expressões completas (!PRESCREVO, !SOLICITO, etc.) com campos obrigatórios/opcionais em colchetes `[]` e parênteses `()`.

- **Parsing Básico**: Regex identifica expressões e extrai campos (ex.: `!PRESCREVO [DIPIRONA] [500MG] [6/6 HORAS]` → JSON: `{"acao": "prescrever", "medicamento": "DIPIRONA", "dosagem": "500MG", "frequencia": "6/6 HORAS"}`).
- **Comandos Padronizados**:
  - !PRESCREVO [MEDICAMENTO] [DOSAGEM] [FREQUÊNCIA] (DURAÇÃO) (AJUSTE);
  - !SOLICITO [TIPO] (URGÊNCIA) (DATA_SUGERIDA) (LOCAL);
  - !ENCAMINHO [ESPECIALIDADE] (MOTIVO) (URGÊNCIA) (DATA_SUGERIDA);
  - !AJUSTO [MEDICAMENTO] [NOVA_DOSAGEM] (MOTIVO);
  - !SUSPENDO [MEDICAMENTO] (MOTIVO) (SUBSTITUTO);
  - !CANCELO [ITEM] (DETALHE) (MOTIVO);
  - !CONSIDERAR [TEXTO];
  - !ORIENTO [TEXTO].
- **Automação Possível**:
  - **Compra de Medicações**: Parser identifica !PRESCREVO e integra com APIs de farmácias para pedidos automáticos ou alertas de estoque.
  - **Trocas de Receitas**: !SUSPENDO notifica renovações ou ajustes digitais.
  - **Auditabilidade por Convênios**: Parseia históricos para reembolsos, rastreando aprovações.
  - **Acompanhamento de Encaminhamentos/Exames**: !ENCAMINHO e !SOLICITO rastreiam status via integrações.
- **Integração de Resultados**: Expandir para [RESULTADOS] com dados de exames parseados.

## Objetivos do Formato
1. **Smart Contracts Médicos**: Arquivos .med como contratos executáveis, com condições, assinaturas e imutabilidade para transações seguras.
2. **Simplicidade e Acessibilidade**: Formato texto puro, editável sem ferramentas complexas.
3. **Automação de Processos**: Parsing de [CONDUTA] para ações condicionais (ex.: lembretes, compras automáticas).
4. **Privacidade**: Hashing de dados sensíveis, controle de acesso via condições.
5. **Escalabilidade**: Parseável em lote para milhares de contratos.
6. **Extensibilidade**: Adaptável para blockchain, DB ou outros sistemas médicos.

### Benefícios
- **Para Médicos**: Reduz tempo em documentação; parsing acelera revisões.
- **Para Pacientes**: Histórico legível, lembretes automáticos.
- **Para Sistemas**: Integração fácil com DBs ou blockchains opcionais para imutabilidade.

## Como Usar
1. Criar smart contract .med: Adicionar seções [PATIENT], [DOCTOR], [TRANSACTION], [CONTRACT_CONDITIONS], [CONTENT].
2. Preencher com dados reais/fictícios; gerar hash e assinatura.
3. Usar comandos padronizados em [CONDUTA] (ex.: !PRESCREVO [DIPIRONA] [500MG] [6/6 HORAS];).
4. Parsear [CONDUTA] com scripts (ex.: Python regex para extrair campos).
5. Validar condições do contrato antes de execução.
6. Integrar com DB/APIs para automação (ex.: notificações de medicação, agendamentos).

## Contribuição
Sugestões para sintaxe ou parsing. Open-source.