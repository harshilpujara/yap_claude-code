const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

function setStatus(text) {
  $("status").textContent = text;
}

// ---------- Live state from the Rust pipeline ----------
const LABELS = { idle: "Idle", recording: "Recording", processing: "Working", error: "Problem" };

function showState({ state, message }) {
  const ind = $("indicator");
  ind.className = "indicator " + state;
  $("indicator-text").textContent = LABELS[state] || state;
  setStatus(message);
}

listen("yapp://state", (e) => showState(e.payload));

listen("yapp://result", (e) => {
  const r = e.payload;
  $("result").hidden = false;
  $("raw").textContent = r.raw || "(no speech detected)";
  $("clean").textContent = r.cleanup_error ? "(cleanup failed - see status above)" : r.clean;
  $("info").textContent = r.info;
});

// ---------- Hotkey capture ----------
const MODIFIER_CODES = new Set([
  "ControlLeft", "ControlRight", "ShiftLeft", "ShiftRight",
  "AltLeft", "AltRight", "MetaLeft", "MetaRight",
]);

function keyName(code) {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit\d$/.test(code)) return code.slice(5);
  if (code.startsWith("Arrow")) return code.slice(5);
  return code; // Space, Enter, F9, ...
}

function hotkeyFromEvent(e) {
  if (MODIFIER_CODES.has(e.code)) return null;
  const isFunctionKey = /^F\d{1,2}$/.test(e.code);
  if (!(e.ctrlKey || e.altKey || e.shiftKey || e.metaKey) && !isFunctionKey) return null;
  const parts = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");
  parts.push(keyName(e.code));
  return parts.join("+");
}

$("hotkey").addEventListener("keydown", (e) => {
  e.preventDefault();
  const combo = hotkeyFromEvent(e);
  if (combo) $("hotkey").value = combo;
});
$("hotkey-reset").addEventListener("click", () => {
  $("hotkey").value = "Ctrl+Space";
});

// ---------- Settings ----------
async function loadSettings() {
  const s = await invoke("get_settings");
  $("hotkey").value = s.hotkey;
  $("hotkey-label").textContent = s.hotkey;
  $("insert-method").value = s.insert_method;
  $("stt-base-url").value = s.stt_base_url;
  $("stt-model").value = s.stt_model;
  $("stt-language").value = s.stt_language;
  $("llm-base-url").value = s.llm_base_url;
  $("llm-model").value = s.llm_model;
  $("vocabulary").value = s.vocabulary;
  $("stt-key").value = "";
  $("llm-key").value = "";
  $("stt-key-note").textContent = s.has_stt_key
    ? "A transcription key is saved."
    : "No transcription key saved yet.";
  $("llm-key-note").textContent = s.has_llm_key
    ? "A separate cleanup key is saved."
    : "No separate cleanup key. The transcription key is used if the service is the same.";
  $("autostart").checked = await invoke("get_autostart");
}

async function showHotkeyStatus() {
  const h = await invoke("get_hotkey_status");
  const box = $("hotkey-error");
  box.hidden = !h.error;
  box.textContent = h.error || "";
}

async function saveSettings({ sttKey = null, llmKey = null } = {}) {
  try {
    await invoke("save_settings", {
      config: {
        stt_base_url: $("stt-base-url").value,
        stt_model: $("stt-model").value,
        stt_language: $("stt-language").value,
        llm_base_url: $("llm-base-url").value,
        llm_model: $("llm-model").value,
        vocabulary: $("vocabulary").value,
        hotkey: $("hotkey").value,
        insert_method: $("insert-method").value,
      },
      sttKey,
      llmKey,
    });
    await loadSettings();
    await showHotkeyStatus();
    setStatus("settings saved - checking models...");
    const check = await invoke("check_models");
    const box = $("model-check");
    box.hidden = false;
    box.classList.toggle("bad", check.warnings.length > 0);
    box.textContent = check.warnings.length
      ? check.warnings.join("\n\n")
      : "Both models were found on the service. All good.";
    setStatus(check.warnings.length ? "settings saved - see model warnings" : "settings saved and verified");
  } catch (e) {
    setStatus("error - " + e);
  }
}

$("save").addEventListener("click", () => {
  const s = $("stt-key").value.trim();
  const l = $("llm-key").value.trim();
  saveSettings({ sttKey: s === "" ? null : s, llmKey: l === "" ? null : l });
});
$("clear-stt-key").addEventListener("click", () => saveSettings({ sttKey: "" }));
$("clear-llm-key").addEventListener("click", () => saveSettings({ llmKey: "" }));

$("autostart").addEventListener("change", async () => {
  const box = $("autostart");
  try {
    await invoke("set_autostart", { enabled: box.checked });
    setStatus(box.checked ? "yapp will start when you sign in" : "start on login turned off");
  } catch (e) {
    box.checked = !box.checked;
    setStatus("error - " + e);
  }
});

// ---------- Typed-text tester ----------
$("test-run").addEventListener("click", async () => {
  const raw = $("test-input").value;
  $("test-output").textContent = "";
  setStatus("cleaning up...");
  try {
    $("test-output").textContent = await invoke("cleanup_text", { raw });
    setStatus("done");
  } catch (e) {
    setStatus("cleanup error - " + e);
  }
});

// ---------- Tabs ----------
function showTab(name) {
  for (const t of ["settings", "insights"]) {
    $("tab-" + t).hidden = t !== name;
    $("tab-btn-" + t).classList.toggle("active", t === name);
  }
  try { localStorage.setItem("yapp-tab", name); } catch (_) {}
  if (name === "insights") loadStats().catch(() => {});
}
document.querySelectorAll(".tab").forEach((b) => b.addEventListener("click", () => showTab(b.dataset.tab)));

// ---------- Insights ----------
const pad2 = (n) => String(n).padStart(2, "0");
const dateKey = (d) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
const fmtInt = (n) => Math.round(n).toLocaleString();

function fmtDuration(seconds) {
  const mins = Math.round(seconds / 60);
  if (mins < 1) return "0m";
  if (mins < 60) return mins + "m";
  return Math.floor(mins / 60) + "h " + (mins % 60) + "m";
}

function renderHeatmap(days) {
  const words = new Map(days.map((d) => [d.date, d.words]));
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  const WEEKS = 27;
  const start = new Date(today);
  start.setDate(today.getDate() - today.getDay() - (WEEKS - 1) * 7); // a Sunday
  const max = Math.max(1, ...words.values());
  const grid = $("heatmap");
  grid.replaceChildren();
  for (let i = 0; i < WEEKS * 7; i++) {
    const d = new Date(start);
    d.setDate(start.getDate() + i);
    const cell = document.createElement("i");
    if (d > today) {
      cell.className = "cell future";
    } else {
      const w = words.get(dateKey(d)) || 0;
      const level = w === 0 ? 0 : Math.min(4, Math.max(1, Math.ceil((w / max) * 4)));
      cell.className = "cell l" + level;
      cell.title = `${d.toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" })}: ${fmtInt(w)} words`;
    }
    grid.appendChild(cell);
  }
}

async function loadStats() {
  const s = await invoke("get_stats");
  $("insights-empty").hidden = s.dictations > 0;
  $("st-words").textContent = fmtInt(s.total_words);
  $("st-wpm").textContent = s.avg_wpm == null ? "\u2014" : fmtInt(s.avg_wpm);
  $("st-count").textContent = fmtInt(s.dictations);
  $("st-saved").textContent = s.dictations > 0 ? fmtDuration(s.time_saved_seconds) : "\u2014";
  $("st-saved-note").textContent = `Compared with typing the same words at about ${s.typing_wpm} words per minute.`;
  $("st-current").textContent = s.current_streak;
  $("st-longest").textContent = s.longest_streak;
  renderHeatmap(s.days);
}

let resetArmed = false;
$("stats-reset").addEventListener("click", async () => {
  if (!resetArmed) {
    resetArmed = true;
    $("stats-reset").textContent = "Click again to erase all stats";
    setTimeout(() => { resetArmed = false; $("stats-reset").textContent = "Reset stats"; }, 4000);
    return;
  }
  resetArmed = false;
  $("stats-reset").textContent = "Reset stats";
  await invoke("reset_stats").catch((e) => setStatus("error - " + e));
});
listen("yapp://stats", () => loadStats().catch(() => {}));

let savedTab = "settings";
try { savedTab = localStorage.getItem("yapp-tab") || "settings"; } catch (_) {}
showTab(savedTab === "insights" ? "insights" : "settings");

loadSettings().catch((e) => setStatus("error - " + e));
// The window is hidden most of the time; re-read everything whenever it is brought up.
window.addEventListener("focus", () => {
  loadSettings().catch(() => {});
  showHotkeyStatus().catch(() => {});
  if (!$("tab-insights").hidden) loadStats().catch(() => {});
});
showHotkeyStatus().catch(() => {});
