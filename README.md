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

- `ui/app.slint`: definizione della GUI Slint.
- `build.rs`: compila `ui/app.slint` durante la build.
- `src/main.rs`: collega i callback della GUI alle operazioni host (`apply`, `backup`, `factory reset`, `time settings`) e aggiorna orologio/timezone.

## Istruzioni di compilazione

Prerequisiti minimi:

- `rustc` / `cargo` (toolchain Rust recente)

Build e run:

```bash
cargo build --release
cargo run
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
