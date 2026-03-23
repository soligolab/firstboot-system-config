const timezoneOptions = [
  "UTC",
  "Europe/Rome",
  "Europe/London",
  "America/New_York",
  "Asia/Tokyo",
];

const fallbackTexts = {
  window_title: "First Boot - User Configuration",
  status_ready: "Ready. Local API backend connected.",
  status_web_ready: "Ready. Web frontend connected to the local backend.",
  status_backend_unreachable: "Local API backend unreachable",
  language_label: "Language",
  clock_date: "Date",
  clock_time: "Time",
  clock_timezone: "Timezone",
  refresh_state: "Refresh status",
  configure_time: "Configure time",
  main_heading: "First boot configuration - users and permissions",
  suggestion_text: "Suggested names: sysadmin (administrator), fieldtech (installer), operator (end user).",
  admin_title: "System administrator",
  installer_title: "Installer",
  viewer_title: "End user",
  admin_description: "Full-access profile for setup and supervision.",
  installer_description: "Field technician for setup, networking and commissioning.",
  viewer_description: "Operator with minimal privileges and read-only access.",
  username_label: "Username",
  full_name_label: "Full name",
  password_label: "Password",
  permissions_label: "Permissions",
  permission_full: "Full administrator",
  permission_network_time: "Networking and system time",
  permission_readonly: "Read only",
  apply_configuration: "Apply configuration",
  backup_recovery: "Backup recovery",
  factory_reset: "Factory reset",
  note_password: "Note: password complexity feedback is informational only (it does not block saving).",
  web_hero_eyebrow: "First boot · web console",
  web_title: "Initial system setup from the browser",
  web_text: "Responsive web interface inspired by industrial HMIs: dark surfaces, cyan accents, strong typography and technical panels. Every action talks to the same local HTTP backend already used by the Slint GUI.",
  host_status_label: "Host status",
  user_profiles_eyebrow: "User profiles",
  user_profiles_heading: "Suggested users and permissions for first boot",
  user_profiles_copy: "Fill in the administrator, installer and end-user profiles. Password feedback is informational and mirrors the Slint GUI.",
  action_helper: "Actions are forwarded to the same endpoints used by the Slint desktop client.",
  output_eyebrow: "Backend output",
  operations_heading: "Operation status",
  log_badge: "Local HTTP",
  time_system_eyebrow: "System time",
  time_modal_title: "Configure date, time and timezone",
  current_date_label: "Current date",
  current_time_label: "Current time",
  format_hint: "Required format: date YYYY-MM-DD, time HH:MM:SS",
  cancel: "Cancel",
  save: "Save",
  password_not_set: "password not set",
  password_weak: "weak password",
  password_fair: "fair password",
  password_strong: "strong password",
  password_feedback_suffix: "informational rating",
};

const userTemplates = [
  {
    key: "admin",
    role: "admin",
    titleKey: "admin_title",
    descriptionKey: "admin_description",
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
    titleKey: "installer_title",
    descriptionKey: "installer_description",
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
    titleKey: "viewer_title",
    descriptionKey: "viewer_description",
    defaults: {
      username: "operator",
      fullName: "End User Operator",
      password: "",
      permissionIdx: 2,
    },
  },
];

const fallbackLanguages = [
  { code: "en", nativeName: "English", flagSvg: null },
  { code: "it", nativeName: "Italiano", flagSvg: null },
  { code: "de", nativeName: "Deutsch", flagSvg: null },
  { code: "es", nativeName: "Español", flagSvg: null },
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
  catalog: null,
  languageCode: "en",
  statusIsDefault: true,
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
  languageLabel: document.querySelector("#languageLabel"),
  languageSelect: document.querySelector("#languageSelect"),
  languageFlag: document.querySelector("#languageFlag"),
  webHeroEyebrow: document.querySelector("#webHeroEyebrow"),
  webTitle: document.querySelector("#webTitle"),
  webText: document.querySelector("#webText"),
  hostStatusLabel: document.querySelector("#hostStatusLabel"),
  clockDateLabel: document.querySelector("#clockDateLabel"),
  clockTimeLabel: document.querySelector("#clockTimeLabel"),
  clockTimezoneLabel: document.querySelector("#clockTimezoneLabel"),
  userProfilesEyebrow: document.querySelector("#userProfilesEyebrow"),
  userProfilesHeading: document.querySelector("#userProfilesHeading"),
  userProfilesCopy: document.querySelector("#userProfilesCopy"),
  actionHelper: document.querySelector("#actionHelper"),
  outputEyebrow: document.querySelector("#outputEyebrow"),
  operationsHeading: document.querySelector("#operationsHeading"),
  logBadge: document.querySelector("#logBadge"),
  timeSystemEyebrow: document.querySelector("#timeSystemEyebrow"),
  timeModalTitle: document.querySelector("#timeModalTitle"),
  currentDateInputLabel: document.querySelector("#currentDateInputLabel"),
  currentTimeInputLabel: document.querySelector("#currentTimeInputLabel"),
  timezoneInputLabel: document.querySelector("#timezoneInputLabel"),
  formatHint: document.querySelector("#formatHint"),
};

void bootstrap();

async function bootstrap() {
  renderTimezoneOptions();
  bindActions();
  await loadLanguageCatalog();
  renderLanguageSelector();
  applyTranslations();
  renderUserCards();
  refreshClock().catch(() => {});
  window.setInterval(() => {
    refreshClock().catch(() => {});
  }, 1000);
}

async function loadLanguageCatalog() {
  try {
    const response = await fetch("/languages.xml");
    if (!response.ok) {
      throw new Error(`Unable to load languages.xml (${response.status})`);
    }

    const xml = await response.text();
    const documentXml = new DOMParser().parseFromString(xml, "application/xml");
    const root = documentXml.querySelector("languages");
    if (!root) {
      throw new Error("Invalid languages XML");
    }

    const languages = Array.from(documentXml.querySelectorAll("language")).map((languageNode) => {
      const texts = Object.fromEntries(
        Array.from(languageNode.querySelectorAll("text")).map((textNode) => [
          textNode.getAttribute("key"),
          textNode.textContent.trim(),
        ]),
      );
      const flagNode = languageNode.querySelector("flag");
      return {
        code: languageNode.getAttribute("code"),
        name: languageNode.getAttribute("name") || languageNode.getAttribute("code"),
        nativeName: languageNode.getAttribute("native-name") || languageNode.getAttribute("name"),
        flagEmoji: flagNode?.getAttribute("emoji") || "",
        flagSvg: flagNode?.textContent?.trim() || null,
        texts,
      };
    });

    state.catalog = {
      defaultCode: root.getAttribute("default") || "en",
      languages,
    };
    state.languageCode = state.catalog.defaultCode;
  } catch (error) {
    console.warn("Failed to load language catalog:", error);
  }
}

function bindActions() {
  elements.refreshClockBtn.addEventListener("click", () => {
    refreshClock().catch(() => {});
  });
  elements.openTimeModalBtn.addEventListener("click", openTimeModal);
  elements.cancelTimeModalBtn.addEventListener("click", closeTimeModal);
  elements.languageSelect.addEventListener("change", onLanguageChange);
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

function renderLanguageSelector() {
  const languages = availableLanguages();
  elements.languageSelect.innerHTML = languages
    .map((language) => `<option value="${escapeAttribute(language.code)}">${escapeHtml(language.nativeName)}</option>`)
    .join("");
  elements.languageSelect.value = state.languageCode;
  updateLanguageFlag();
}

function onLanguageChange(event) {
  state.languageCode = event.currentTarget.value;
  applyTranslations();
  renderUserCards();
}

function applyTranslations() {
  document.documentElement.lang = state.languageCode;
  document.title = t("window_title");
  elements.languageLabel.textContent = t("language_label");
  elements.languageSelect.setAttribute("aria-label", t("language_label"));
  elements.webHeroEyebrow.textContent = t("web_hero_eyebrow");
  elements.webTitle.textContent = t("web_title");
  elements.webText.textContent = t("web_text");
  elements.hostStatusLabel.textContent = t("host_status_label");
  elements.clockDateLabel.textContent = t("clock_date");
  elements.clockTimeLabel.textContent = t("clock_time");
  elements.clockTimezoneLabel.textContent = t("clock_timezone");
  elements.refreshClockBtn.textContent = t("refresh_state");
  elements.openTimeModalBtn.textContent = t("configure_time");
  elements.userProfilesEyebrow.textContent = t("user_profiles_eyebrow");
  elements.userProfilesHeading.textContent = t("user_profiles_heading");
  elements.userProfilesCopy.textContent = t("user_profiles_copy");
  elements.applyConfigurationBtn.textContent = t("apply_configuration");
  elements.backupRecoveryBtn.textContent = t("backup_recovery");
  elements.factoryResetBtn.textContent = t("factory_reset");
  elements.actionHelper.textContent = t("action_helper");
  elements.outputEyebrow.textContent = t("output_eyebrow");
  elements.operationsHeading.textContent = t("operations_heading");
  elements.logBadge.textContent = t("log_badge");
  elements.timeSystemEyebrow.textContent = t("time_system_eyebrow");
  elements.timeModalTitle.textContent = t("time_modal_title");
  elements.currentDateInputLabel.textContent = t("current_date_label");
  elements.currentTimeInputLabel.textContent = t("current_time_label");
  elements.timezoneInputLabel.textContent = t("clock_timezone");
  elements.formatHint.textContent = `${t("format_hint")}.`;
  elements.cancelTimeModalBtn.textContent = t("cancel");
  elements.saveTimeSettingsBtn.textContent = t("save");

  if (state.statusIsDefault) {
    elements.statusMessage.textContent = t("status_web_ready");
  }

  updateLanguageFlag();
}

function renderTimezoneOptions() {
  elements.timeTimezoneInput.innerHTML = timezoneOptions
    .map((timezone) => `<option value="${escapeHtml(timezone)}">${escapeHtml(timezone)}</option>`)
    .join("");
}

function renderUserCards() {
  const permissionOptions = [
    t("permission_full"),
    t("permission_network_time"),
    t("permission_readonly"),
  ];

  elements.userGrid.innerHTML = userTemplates
    .map((template) => {
      const user = state.users[template.key];
      const title = t(template.titleKey);
      const description = t(template.descriptionKey);
      return `
        <article class="user-card">
          <p class="eyebrow">${escapeHtml(template.role)}</p>
          <h3>${escapeHtml(title)}</h3>
          <p>${escapeHtml(description)}</p>
          <div class="form-grid">
            <label>
              <span>${escapeHtml(t("username_label"))}</span>
              <input data-field="username" data-user-key="${template.key}" value="${escapeAttribute(user.username)}" autocomplete="off">
            </label>
            <label>
              <span>${escapeHtml(t("full_name_label"))}</span>
              <input data-field="fullName" data-user-key="${template.key}" value="${escapeAttribute(user.fullName)}" autocomplete="off">
            </label>
            <label>
              <span>${escapeHtml(t("password_label"))}</span>
              <input data-field="password" data-user-key="${template.key}" type="password" value="${escapeAttribute(user.password)}" autocomplete="new-password">
            </label>
            <p class="feedback" id="feedback-${template.key}">${escapeHtml(passwordFeedback(title, user.password))}</p>
            <label>
              <span>${escapeHtml(t("permissions_label"))}</span>
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
      feedback.textContent = passwordFeedback(t(template.titleKey), control.value);
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
    state.statusIsDefault = false;
    setStatus(`${t("status_backend_unreachable")}: ${error.message}`);
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
  state.statusIsDefault = false;
  elements.statusMessage.textContent = message;
}

function passwordFeedback(role, password) {
  if (!password) {
    return `${role}: ${t("password_not_set")}.`;
  }

  let score = 0;
  if (password.length >= 12) score += 1;
  if (/[a-z]/.test(password)) score += 1;
  if (/[A-Z]/.test(password)) score += 1;
  if (/\d/.test(password)) score += 1;
  if (/[^A-Za-z0-9]/.test(password)) score += 1;

  const message = score <= 2 ? t("password_weak") : score <= 4 ? t("password_fair") : t("password_strong");
  return `${role}: ${message} (${t("password_feedback_suffix")}).`;
}

function availableLanguages() {
  return state.catalog?.languages?.length ? state.catalog.languages : fallbackLanguages;
}

function currentLanguage() {
  return availableLanguages().find((language) => language.code === state.languageCode) || availableLanguages()[0];
}

function t(key) {
  return currentLanguage()?.texts?.[key] || fallbackTexts[key] || key;
}

function updateLanguageFlag() {
  const language = currentLanguage();
  if (language?.flagSvg) {
    elements.languageFlag.src = svgToDataUri(language.flagSvg);
    elements.languageFlag.alt = `${language.nativeName} flag`;
    elements.languageFlag.classList.remove("language-flag-hidden");
  } else {
    elements.languageFlag.removeAttribute("src");
    elements.languageFlag.alt = "";
    elements.languageFlag.classList.add("language-flag-hidden");
  }
}

function svgToDataUri(svg) {
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
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
