# First Boot System Config

Applicazione desktop in **Rust + Slint** per guidare la configurazione iniziale di un sistema embedded/industrial con gestione utenti, permessi e impostazioni orarie.

## Anteprima GUI

![Anteprima GUI](docs/gui-preview.svg)

## Cosa fa

- Configura tre profili utente suggeriti (amministratore, installatore, utente finale).
- Permette di assegnare livelli di permesso differenti per ciascun utente.
- Mostra un feedback di complessità password (informativo).
- Include hook placeholder per:
  - applicazione configurazione utenti,
  - backup recovery,
  - factory reset.
- Mostra in alto **data, ora e timezone correnti**.
- Include il pulsante **"Configura orario"** con pagina/modale dedicata per:
  - modifica data,
  - modifica ora,
  - selezione timezone,
  - salvataggio (hook verso futura integrazione con OS).

## Istruzioni di compilazione

Prerequisiti minimi:

- `rustc` / `cargo` (toolchain Rust recente)
- compilatore C/C++ (`build-essential`)
- librerie base grafiche per Linux desktop

Build e run:

```bash
cargo build --release
cargo run
```

## Setup sintetico ambiente Rust + Slint su Linux (Debian/Ubuntu)

### 1) Installare dipendenze di sistema

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  curl \
  pkg-config \
  libx11-dev libxext-dev libxrender-dev libxfixes-dev \
  libxcb1-dev libxkbcommon-dev libwayland-dev wayland-protocols
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

### 4) Clonare e avviare il progetto

```bash
git clone <url-repository>
cd firstboot-system-config
cargo run
```

> Nota: in ambienti headless o container è necessario un backend grafico (X11/Wayland) per visualizzare la finestra Slint.
