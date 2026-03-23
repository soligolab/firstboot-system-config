const permissionOptions = [
  "Amministratore completo",
  "Rete e ora di sistema",
  "Sola visualizzazione",
];

const timezoneOptions = [
  "UTC",
  "Europe/Rome",
  "Europe/London",
  "America/New_York",
  "Asia/Tokyo",
];

const userTemplates = [
  {
    key: "admin",
    role: "admin",
    title: "Amministratore di sistema",
    description: "Profilo ad accesso completo per configurazione e supervisione.",
    defaults: {
      username: "sysadmin",
      fullName: "System Administrator",
      password: "",
      permissionIdx: 0,
    },
  },
  {
    key: "installer",
    role: "installer",
    title: "Installatore",
    description: "Tecnico di campo per setup, rete e messa in servizio.",
    defaults: {
      username: "fieldtech",
      fullName: "Field Installer",
      password: "",
      permissionIdx: 1,
    },
  },
  {
    key: "viewer",
    role: "viewer",
    title: "Utente finale",
    description: "Operatore con privilegi minimi e accesso di sola consultazione.",
    defaults: {
      username: "operator",
      fullName: "End User Operator",
      password: "",
      permissionIdx: 2,
    },
  },
];

const state = {
  time: {
    date: "----/--/--",
    time: "--:--:--",
    timezone: "UTC",
  },
  users: Object.fromEntries(
    userTemplates.map((template) => [template.key, { ...template.defaults }]),
  ),
};

const elements = {
  userGrid: document.querySelector("#userGrid"),
  statusMessage: document.querySelector("#statusMessage"),
  currentDate: document.querySelector("#currentDate"),
  currentTime: document.querySelector("#currentTime"),
  currentTimezone: document.querySelector("#currentTimezone"),
  refreshClockBtn: document.querySelector("#refreshClockBtn"),
  openTimeModalBtn: document.querySelector("#openTimeModalBtn"),
  applyConfigurationBtn: document.querySelector("#applyConfigurationBtn"),
  backupRecoveryBtn: document.querySelector("#backupRecoveryBtn"),
  factoryResetBtn: document.querySelector("#factoryResetBtn"),
  timeModalBackdrop: document.querySelector("#timeModalBackdrop"),
  timeDateInput: document.querySelector("#timeDateInput"),
  timeTimeInput: document.querySelector("#timeTimeInput"),
  timeTimezoneInput: document.querySelector("#timeTimezoneInput"),
  cancelTimeModalBtn: document.querySelector("#cancelTimeModalBtn"),
  saveTimeSettingsBtn: document.querySelector("#saveTimeSettingsBtn"),
};

bootstrap();

function bootstrap() {
  renderTimezoneOptions();
  renderUserCards();
  bindActions();
  refreshClock().catch(() => {});
  window.setInterval(() => {
    refreshClock().catch(() => {});
  }, 1000);
}

function renderTimezoneOptions() {
  elements.timeTimezoneInput.innerHTML = timezoneOptions
    .map((timezone) => `<option value="${escapeHtml(timezone)}">${escapeHtml(timezone)}</option>`)
    .join("");
}

function renderUserCards() {
  elements.userGrid.innerHTML = userTemplates
    .map((template) => {
      const user = state.users[template.key];
      return `
        <article class="user-card">
          <p class="eyebrow">${escapeHtml(template.role)}</p>
          <h3>${escapeHtml(template.title)}</h3>
          <p>${escapeHtml(template.description)}</p>
          <div class="form-grid">
            <label>
              <span>Username</span>
              <input data-field="username" data-user-key="${template.key}" value="${escapeAttribute(user.username)}" autocomplete="off">
            </label>
            <label>
              <span>Nome esteso</span>
              <input data-field="fullName" data-user-key="${template.key}" value="${escapeAttribute(user.fullName)}" autocomplete="off">
            </label>
            <label>
              <span>Password</span>
              <input data-field="password" data-user-key="${template.key}" type="password" value="${escapeAttribute(user.password)}" autocomplete="new-password">
            </label>
            <p class="feedback" id="feedback-${template.key}">${escapeHtml(passwordFeedback(template.title, user.password))}</p>
            <label>
              <span>Permessi</span>
              <select data-field="permissionIdx" data-user-key="${template.key}">
                ${permissionOptions
                  .map((option, idx) => `<option value="${idx}" ${idx === user.permissionIdx ? "selected" : ""}>${escapeHtml(option)}</option>`)
                  .join("")}
              </select>
            </label>
          </div>
        </article>
      `;
    })
    .join("");

  elements.userGrid.querySelectorAll("input, select").forEach((control) => {
    control.addEventListener("input", onUserFieldChange);
    control.addEventListener("change", onUserFieldChange);
  });
}

function bindActions() {
  elements.refreshClockBtn.addEventListener("click", () => {
    refreshClock().catch(() => {});
  });
  elements.openTimeModalBtn.addEventListener("click", openTimeModal);
  elements.cancelTimeModalBtn.addEventListener("click", closeTimeModal);
  elements.timeModalBackdrop.addEventListener("click", (event) => {
    if (event.target === elements.timeModalBackdrop) {
      closeTimeModal();
    }
  });
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !elements.timeModalBackdrop.classList.contains("hidden")) {
      closeTimeModal();
    }
  });

  elements.applyConfigurationBtn.addEventListener("click", async () => {
    await runAction(elements.applyConfigurationBtn, async () => {
      const body = userTemplates
        .map((template) => {
          const user = state.users[template.key];
          return [template.role, user.username, user.fullName, user.password, user.permissionIdx].join("|");
        })
        .join("\n");
      return sendRequest("/api/configuration", { method: "POST", body });
    });
  });

  elements.backupRecoveryBtn.addEventListener("click", async () => {
    await runAction(elements.backupRecoveryBtn, () =>
      sendRequest("/api/backup-recovery", { method: "POST", body: "" }),
    );
  });

  elements.factoryResetBtn.addEventListener("click", async () => {
    await runAction(elements.factoryResetBtn, () =>
      sendRequest("/api/factory-reset", { method: "POST", body: "" }),
    );
  });

  elements.saveTimeSettingsBtn.addEventListener("click", async () => {
    await runAction(elements.saveTimeSettingsBtn, async () => {
      const timeValue = normalizeTime(elements.timeTimeInput.value);
      const payload = [
        `date=${elements.timeDateInput.value}`,
        `time=${timeValue}`,
        `timezone=${elements.timeTimezoneInput.value}`,
        "",
      ].join("\n");
      const response = await sendRequest("/api/time", { method: "POST", body: payload });
      closeTimeModal();
      await refreshClock();
      return response;
    });
  });
}

function onUserFieldChange(event) {
  const control = event.currentTarget;
  const userKey = control.dataset.userKey;
  const field = control.dataset.field;
  const user = state.users[userKey];

  if (!user || !field) {
    return;
  }

  user[field] = field === "permissionIdx" ? Number(control.value) : control.value;

  if (field === "password") {
    const template = userTemplates.find((entry) => entry.key === userKey);
    const feedback = document.querySelector(`#feedback-${userKey}`);
    if (template && feedback) {
      feedback.textContent = passwordFeedback(template.title, control.value);
    }
  }
}

async function refreshClock() {
  try {
    const body = await sendRequest("/api/time", { method: "GET" }, false);
    state.time = parseKeyValueBody(body);
    renderClock();
    return body;
  } catch (error) {
    setStatus(`Backend API non raggiungibile: ${error.message}`);
    throw error;
  }
}

function renderClock() {
  elements.currentDate.textContent = state.time.date || "----/--/--";
  elements.currentTime.textContent = state.time.time || "--:--:--";
  elements.currentTimezone.textContent = state.time.timezone || "UTC";
}

function openTimeModal() {
  elements.timeDateInput.value = normalizeDate(state.time.date);
  elements.timeTimeInput.value = normalizeTime(state.time.time);
  elements.timeTimezoneInput.value = timezoneOptions.includes(state.time.timezone)
    ? state.time.timezone
    : "UTC";
  elements.timeModalBackdrop.classList.remove("hidden");
  elements.timeModalBackdrop.setAttribute("aria-hidden", "false");
}

function closeTimeModal() {
  elements.timeModalBackdrop.classList.add("hidden");
  elements.timeModalBackdrop.setAttribute("aria-hidden", "true");
}

async function runAction(button, action) {
  button.disabled = true;
  try {
    const response = await action();
    setStatus(response);
  } catch (error) {
    setStatus(`ERROR: ${error.message}`);
  } finally {
    button.disabled = false;
  }
}

async function sendRequest(url, options, updateStatus = true) {
  const response = await fetch(url, {
    ...options,
    headers: {
      "Content-Type": "text/plain; charset=utf-8",
      ...(options.headers || {}),
    },
  });

  const text = await response.text();
  if (!response.ok) {
    throw new Error(`${response.status} ${response.statusText}: ${text}`);
  }

  if (updateStatus) {
    setStatus(text);
  }

  return text;
}

function setStatus(message) {
  elements.statusMessage.textContent = message;
}

function passwordFeedback(role, password) {
  if (!password) {
    return `${role}: password non impostata.`;
  }

  let score = 0;
  if (password.length >= 12) score += 1;
  if (/[a-z]/.test(password)) score += 1;
  if (/[A-Z]/.test(password)) score += 1;
  if (/\d/.test(password)) score += 1;
  if (/[^A-Za-z0-9]/.test(password)) score += 1;

  const message = score <= 2 ? "password debole" : score <= 4 ? "password discreta" : "password forte";
  return `${role}: ${message} (valutazione informativa).`;
}

function parseKeyValueBody(body) {
  return body
    .split(/\n+/)
    .filter(Boolean)
    .reduce((acc, line) => {
      const [key, ...rest] = line.split("=");
      if (key && rest.length > 0) {
        acc[key] = rest.join("=");
      }
      return acc;
    }, {});
}

function normalizeDate(value) {
  return /^\d{4}-\d{2}-\d{2}$/.test(value) ? value : "";
}

function normalizeTime(value) {
  if (/^\d{2}:\d{2}:\d{2}$/.test(value)) {
    return value;
  }
  if (/^\d{2}:\d{2}$/.test(value)) {
    return `${value}:00`;
  }
  return "00:00:00";
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function escapeAttribute(value) {
  return escapeHtml(value);
}
