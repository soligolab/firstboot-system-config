# First Boot System Config

Applicazione desktop in **Rust + Slint** per guidare la configurazione iniziale di un sistema embedded/industrial con gestione utenti, permessi e impostazioni orarie.

## Anteprima GUI

![Anteprima GUI](docs/gui-preview.svg)

## Cosa fa

- Configura tre profili utente suggeriti (amministratore, installatore, utente finale).
- Permette di assegnare livelli di permesso differenti per ciascun utente.
- Mostra un feedback di complessità password con barra colorata (informativo).
- Esegue azioni lato host (backend Rust) per apply, backup recovery, factory reset e aggiornamento data/ora/timezone.
- Mostra in alto **data, ora e timezone correnti**.
- Usa come GUI runtime il file `ui/app.slint`, compilato in fase di build tramite `slint-build`.
- Mostra a video l'esito delle operazioni host direttamente nella finestra Slint.

## Architettura attuale

La GUI Slint e il backend host sono ora separati tramite una **API HTTP locale**:

- `ui/app.slint`: definizione della GUI Slint condivisa.
- `src/main.rs`: avvia la GUI Slint e la collega a un `ApiClient` HTTP, invece che ai comandi host diretti.
- `src/api.rs`: espone il backend API locale e il client HTTP usato dalla GUI.
- `src/backend.rs`: implementa il servizio host nativo che esegue le operazioni di sistema.
- `src/models.rs`: modelli condivisi tra frontend e backend API.
- `build.rs`: compila `ui/app.slint` durante la build.

In modalità GUI, l'app avvia automaticamente un backend locale su `127.0.0.1:7878` (configurabile con `FIRSTBOOT_API_ADDR`) e la GUI Slint comunica solo via HTTP verso gli endpoint locali.

### Endpoint disponibili

- `GET /api/time`
- `POST /api/time`
- `POST /api/configuration`
- `POST /api/backup-recovery`
- `POST /api/factory-reset`

Questa separazione prepara il progetto a riusare la stessa UI Slint con un backend API stabile anche per futuri frontend via browser/URL.

## Istruzioni di compilazione

Prerequisiti minimi:

- `rustc` / `cargo` (toolchain Rust recente)

Build e run:

```bash
cargo build --release
cargo run
```

Solo backend API:

```bash
cargo run -- server
```

Aprendo `http://127.0.0.1:7878/` nel browser non viene mostrata la GUI Slint: la porta espone il backend HTTP locale usato dalla GUI nativa. La root (`/`) ora mostra una pagina informativa, mentre i dati utili sono sugli endpoint `/api/...` (ad esempio `GET /api/time`).

Con indirizzo API personalizzato:

```bash
FIRSTBOOT_API_ADDR=0.0.0.0:7878 cargo run -- server
```

## Setup sintetico ambiente Rust su Linux (Debian/Ubuntu)

### 1) Installare dipendenze di base

```bash
sudo apt update
sudo apt install -y build-essential curl
```

### 2) Installare Rust (rustup)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup update stable
```

### 3) Verifica toolchain

```bash
rustc --version
cargo --version
```
