# First Boot System Config

Applicazione desktop in **Rust + Slint** per guidare la configurazione iniziale di un sistema embedded/industrial con gestione utenti, permessi e impostazioni orarie.

## Anteprima GUI

![Anteprima GUI](docs/gui-preview.svg)

## Obiettivo del tool

Il progetto fornisce una GUI desktop nativa per eseguire le attività tipiche di **first boot**:

- preparazione di utenti e permessi iniziali;
- verifica/modifica di data, ora e timezone;
- attivazione di operazioni host come apply configuration, backup recovery e factory reset;
- visualizzazione immediata dell'esito delle operazioni direttamente nell'interfaccia.

> La GUI non chiama direttamente i comandi di sistema: dialoga sempre con un backend HTTP locale incluso nel progetto.

## Architettura in breve

Il repository è organizzato in due blocchi principali:

- **Frontend nativo Slint**: definito in `ui/app.slint` e compilato in build-time con `slint-build`.
- **Backend HTTP locale**: implementato in Rust e avviato sulla porta `127.0.0.1:7878` per default.

### Componenti principali

- `src/main.rs`: bootstrap dell'applicazione, avvio GUI e collegamento dei callback Slint al client API.
- `src/api.rs`: server HTTP minimale locale + client HTTP raw usato dalla GUI.
- `src/backend.rs`: implementazione demo del servizio host che esegue i comandi locali.
- `src/models.rs`: payload e modelli condivisi tra frontend e backend.
- `ui/app.slint`: layout, proprietà e callback della GUI.
- `build.rs`: compilazione della UI Slint durante la build.

## Come usare il tool

### Modalità 1: GUI completa

Questa è la modalità normale per l'operatore:

```bash
cargo run
```

Cosa succede:

1. il binario avvia un backend HTTP locale in background;
2. la finestra Slint viene aperta;
3. la GUI legge data/ora/timezone dal backend;
4. tutte le azioni dei pulsanti vengono inoltrate via HTTP locale.

### Modalità 2: solo backend API

Utile per test manuali, debugging o integrazione con altri client:

```bash
cargo run -- server
```

In questa modalità non si apre la GUI: viene esposto soltanto il server HTTP locale.

### Cambiare indirizzo/porta API

Per usare un binding diverso:

```bash
FIRSTBOOT_API_ADDR=0.0.0.0:7878 cargo run -- server
```

Oppure, per la modalità GUI:

```bash
FIRSTBOOT_API_ADDR=127.0.0.1:7879 cargo run
```

## Flusso operativo dalla GUI

Quando avvii l'applicazione puoi seguire questo flusso:

1. **Controlla data, ora e timezone** mostrate nella barra superiore.
2. **Compila i tre profili utente** suggeriti:
   - amministratore;
   - installatore;
   - utente finale.
3. **Inserisci le password**: il feedback di robustezza è informativo e non blocca il salvataggio.
4. **Scegli il livello di permesso** da ciascun menu a tendina.
5. Premi **Applica configurazione** per inviare i dati al backend.
6. Usa **Backup recovery** o **Factory Reset** se vuoi testare gli altri flussi host.
7. Se devi cambiare l'orario, premi **Configura orario**, modifica i valori e poi **Salva**.

## Cosa fanno davvero i pulsanti oggi

L'implementazione attuale del backend è volutamente dimostrativa:

- **Applica configurazione**: serializza i tre utenti e li appende a `/tmp/firstboot-user-config.log`.
- **Backup recovery**: scrive un evento in `/tmp/firstboot-actions.log` ed esegue `uname -a`.
- **Factory Reset**: scrive un evento in `/tmp/firstboot-actions.log` ed esegue `date`.
- **Salva orario**: prova a invocare `timedatectl set-timezone` e `timedatectl set-time`.

Questo significa che il progetto è già utile come base per UI e integrazione, ma il layer host può essere sostituito in seguito con logiche reali di provisioning del dispositivo.

## Endpoint HTTP disponibili

Il backend locale espone questi endpoint:

- `GET /`
- `GET /api/time`
- `POST /api/time`
- `POST /api/configuration`
- `POST /api/backup-recovery`
- `POST /api/factory-reset`

### Esempi rapidi con `curl`

#### Leggere lo stato orario corrente

```bash
curl http://127.0.0.1:7878/api/time
```

Risposta tipica:

```text
date=2026-03-22
time=10:30:15
timezone=Europe/Rome
```

#### Inviare una modifica di data/ora/timezone

```bash
curl -X POST http://127.0.0.1:7878/api/time \
  -H 'Content-Type: text/plain' \
  --data-binary $'date=2026-03-22\ntime=10:35:00\ntimezone=UTC\n'
```

#### Inviare la configurazione utenti

```bash
curl -X POST http://127.0.0.1:7878/api/configuration \
  -H 'Content-Type: text/plain' \
  --data-binary $'admin|sysadmin|System Administrator|Secret123!|0\ninstaller|fieldtech|Field Installer|Install123!|1\nviewer|operator|End User Operator|View123!|2'
```

#### Testare backup recovery

```bash
curl -X POST http://127.0.0.1:7878/api/backup-recovery
```

#### Testare factory reset

```bash
curl -X POST http://127.0.0.1:7878/api/factory-reset
```

## Formato dei payload

### `/api/time`

Formato testuale `key=value`:

```text
date=YYYY-MM-DD
time=HH:MM:SS
timezone=Area/City
```

### `/api/configuration`

Una riga per utente, nel formato:

```text
role|username|full_name|password|permission_idx
```

Dove `permission_idx` corrisponde alle voci del `ComboBox` in UI:

- `0` = amministratore completo;
- `1` = rete e ora di sistema;
- `2` = sola visualizzazione.

## Compilazione

### Prerequisiti minimi

- `rustc`
- `cargo`
- toolchain Rust stabile recente

### Build e run

```bash
cargo build --release
cargo run
```

## Setup rapido ambiente Rust su Debian/Ubuntu

### 1. Installare dipendenze di base

```bash
sudo apt update
sudo apt install -y build-essential curl
```

### 2. Installare Rust con rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup update stable
```

### 3. Verifica toolchain

```bash
rustc --version
cargo --version
```

## Note utili per sviluppatori

- La GUI usa `slint::include_modules!()` per importare il codice generato da `build.rs`.
- La comunicazione HTTP è intenzionalmente minimale e non usa framework esterni.
- Il backend locale usa file in `/tmp` per simulare effetti persistenti senza toccare configurazioni reali di sistema, tranne quando si invoca il salvataggio dell'orario con `timedatectl`.
- Se apri `http://127.0.0.1:7878/` dal browser vedrai solo una pagina informativa, non la GUI desktop.
