# First Boot System Config

Applicazione di **first boot** per sistemi embedded/industriali con due front-end complementari:

- **GUI nativa desktop** realizzata in **Rust + Slint**;
- **WebPage responsive** servita direttamente dal backend Rust sulla stessa API locale.

L'obiettivo del progetto è accompagnare l'operatore nelle attività iniziali di provisioning del dispositivo: configurazione utenti, permessi, data/ora/timezone e avvio di azioni host come apply configuration, backup recovery e factory reset.

## Anteprima GUI

![Anteprima GUI](docs/gui-preview.svg)

---

## Getting Started

### 1. Cosa avvia davvero il progetto

Il binario espone sempre un **backend HTTP locale**. A seconda di come lo lanci puoi usare:

- la **GUI nativa**, pensata per l'uso su dispositivo o postazione locale;
- la **WebPage**, utile quando vuoi operare da browser usando gli stessi endpoint.

In pratica l'architettura è questa:

1. il backend Rust espone API locali su `127.0.0.1:7878` di default;
2. la GUI Slint usa quelle API per leggere stato e inviare azioni;
3. anche la WebPage usa le stesse API e viene servita dallo stesso processo Rust.

> La GUI non esegue direttamente i comandi di sistema: tutte le operazioni passano dal backend HTTP locale.

### 2. Avvio rapido

#### Avviare GUI nativa + backend

```bash
cargo run
```

Questa è la modalità consigliata per il primo utilizzo. Il processo:

1. avvia il backend locale in background;
2. apre la finestra desktop Slint;
3. sincronizza data, ora e timezone dalla macchina host;
4. rende disponibile in parallelo anche la WebPage sulla root HTTP locale.

#### Avviare solo backend + WebPage

```bash
cargo run -- server
```

Questa modalità è utile quando:

- vuoi usare solo il browser;
- stai facendo debug o test degli endpoint;
- vuoi integrare client esterni via HTTP locale.

### 3. Aprire la WebPage

Con il server attivo, apri nel browser:

```text
http://127.0.0.1:7878/
```

Gli asset della WebPage sono serviti direttamente dal backend ai path:

- `GET /`
- `GET /app.css`
- `GET /app.js`

### 4. Cambiare host/porta API

Per cambiare bind address usa la variabile d'ambiente `FIRSTBOOT_API_ADDR`.

Esempio backend headless:

```bash
FIRSTBOOT_API_ADDR=0.0.0.0:7878 cargo run -- server
```

Esempio GUI desktop:

```bash
FIRSTBOOT_API_ADDR=127.0.0.1:7879 cargo run
```

---

## Le due interfacce: quando usare GUI nativa e quando usare WebPage

## GUI nativa desktop (Rust + Slint)

La **GUI nativa** è l'interfaccia principale per l'operatore locale. È adatta quando il dispositivo ha monitor/touch locale oppure quando vuoi un'esperienza desktop dedicata.

### Cosa offre

- barra superiore con **data, ora e timezone** aggiornate periodicamente;
- form guidato per i tre profili utente:
  - amministratore;
  - installatore;
  - utente finale;
- feedback password immediato;
- selezione del livello di permesso da menu a tendina;
- popup dedicato per **configurare data, ora e timezone**;
- pulsanti per:
  - **Applica configurazione**;
  - **Backup recovery**;
  - **Factory reset**.

### Quando preferirla

Usa la GUI nativa se:

- lavori direttamente sul dispositivo;
- vuoi un'interfaccia più “appliance-like”;
- ti serve un flusso operatore lineare e focalizzato;
- vuoi ridurre al minimo il contesto tecnico visibile all'utente finale.

### Flusso operativo consigliato nella GUI

1. Verifica **data, ora e timezone** nella toolbar superiore.
2. Compila i tre profili utente richiesti.
3. Inserisci le password e controlla il feedback di robustezza.
4. Seleziona il livello di permesso per ogni profilo.
5. Premi **Applica configurazione**.
6. Se necessario usa **Configura orario** per correggere data/ora/timezone.
7. Esegui **Backup recovery** o **Factory reset** solo quando vuoi testare o simulare quei flussi.

---

## WebPage responsive

La **WebPage** è la seconda interfaccia del progetto. Non è una pagina informativa: è una vera UI operativa che usa gli stessi endpoint della GUI nativa.

### Cosa offre

- dashboard iniziale con **stato host** e metriche di data/ora/timezone;
- layout responsive, adatto anche a schermi più piccoli;
- sezione utenti con gli stessi tre profili della GUI;
- feedback password coerente con il client desktop;
- modal per configurare data, ora e timezone;
- area stato/output backend per leggere subito l'esito delle operazioni;
- pulsanti per apply configuration, backup recovery e factory reset.

### Quando preferirla

Usa la WebPage se:

- vuoi accedere all'interfaccia da browser senza aprire la GUI desktop;
- stai facendo demo, collaudo o supporto remoto su rete locale;
- vuoi verificare rapidamente il comportamento delle API con una UI già pronta;
- preferisci un'interfaccia responsive servita direttamente dal backend.

### Flusso operativo consigliato nella WebPage

1. Apri `http://127.0.0.1:7878/`.
2. Controlla il riquadro **Stato host**.
3. Premi **Aggiorna stato** se vuoi forzare una nuova lettura.
4. Compila i profili utente nella sezione centrale.
5. Usa **Configura orario** per aprire la finestra modale e modificare data/ora/timezone.
6. Premi **Applica configurazione** o gli altri pulsanti operativi.
7. Controlla il pannello **Stato operazioni** per l'esito restituito dal backend.

---

## Differenze pratiche tra GUI nativa e WebPage

| Aspetto | GUI nativa | WebPage |
|---|---|---|
| Tecnologia | Slint desktop | HTML/CSS/JS serviti dal backend |
| Avvio | `cargo run` | `cargo run -- server` oppure `cargo run` + browser |
| Uso tipico | operatore locale sul dispositivo | accesso da browser, test, demo |
| UX | più “dedicata” e desktop | più flessibile e responsive |
| API usate | API HTTP locali | le stesse API HTTP locali |

In termini funzionali, le due interfacce sono allineate: cambia soprattutto il contesto d'uso.

---

## Novità recenti

Questa è la sezione più utile per capire **cosa è cambiato nelle ultime revisioni** del progetto.

### Ultime modifiche introdotte

- è stata aggiunta una **WebPage responsive completa**, servita direttamente dal backend Rust;
- la root `GET /` ora espone la UI web invece di una semplice pagina placeholder/informativa;
- GUI nativa e WebPage condividono lo stesso backend HTTP locale e gli stessi endpoint;
- la documentazione e i commenti del codice sono stati migliorati per chiarire architettura, callback e payload;
- il routing del backend è stato sistemato per servire correttamente la UI web e gli asset associati.

### Impatto pratico delle ultime modifiche

Oggi il progetto può essere usato in due modi reali:

1. come **app desktop nativa** per l'operatore locale;
2. come **console web** per browser, utile per test, demo e accesso più flessibile.

Questo rende il repository molto più vicino a una base di prodotto: non c'è più una sola GUI, ma un backend locale condiviso con due canali di accesso coerenti.

---

## Endpoint HTTP disponibili

Il backend locale espone questi endpoint:

- `GET /`
- `GET /app.css`
- `GET /app.js`
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

---

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

Dove `permission_idx` corrisponde alle voci del selettore permessi in UI:

- `0` = amministratore completo;
- `1` = rete e ora di sistema;
- `2` = sola visualizzazione.

---

## Cosa fanno davvero i pulsanti oggi

L'implementazione attuale del backend è ancora dimostrativa, ma già utile per testare il flusso end-to-end.

- **Applica configurazione**: serializza i tre utenti e li appende a `/tmp/firstboot-user-config.log`.
- **Backup recovery**: scrive un evento in `/tmp/firstboot-actions.log` ed esegue `uname -a`.
- **Factory reset**: scrive un evento in `/tmp/firstboot-actions.log` ed esegue `date`.
- **Salva orario**: prova a invocare `timedatectl set-timezone` e `timedatectl set-time`.

Questo significa che il progetto è già utile come base di UX, integrazione e test, mentre il layer host può essere sostituito in seguito con logiche reali di provisioning.

---

## Architettura del repository

### Componenti principali

- `src/main.rs`: bootstrap dell'applicazione, avvio GUI e collegamento dei callback Slint al client API.
- `src/api.rs`: server HTTP locale minimale + client HTTP raw usato dalla GUI.
- `src/web.rs` + `web/`: asset e markup della WebPage servita dal backend.
- `src/backend.rs`: implementazione demo del servizio host che esegue i comandi locali.
- `src/models.rs`: payload e modelli condivisi tra frontend e backend.
- `ui/app.slint`: layout, proprietà e callback della GUI nativa.
- `build.rs`: compilazione della UI Slint durante la build.

---

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

---

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

---

## Note utili per sviluppatori

- La GUI usa `slint::include_modules!()` per importare il codice generato da `build.rs`.
- La comunicazione HTTP è volutamente minimale e non usa framework esterni.
- Il backend locale usa file in `/tmp` per simulare effetti persistenti senza toccare configurazioni reali di sistema, tranne quando si invoca il salvataggio dell'orario con `timedatectl`.
- La WebPage e la GUI nativa sono due front-end distinti, ma condividono la stessa logica di backend e lo stesso perimetro funzionale.
