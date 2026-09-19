const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

let lastPath = null;

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
  lastPath = r.recording_path;
  $("result").hidden = false;
  $("raw").textContent = r.raw || "(no speech detected)";
  $("clean").textContent = r.cleanup_error ? "(cleanup failed - see status above)" : r.clean;
  $("info").textContent = r.info;
});

$("reveal").addEventListener("click", () => {
  if (lastPath) invoke("reveal_recording", { path: lastPath }).catch((e) => setStatus("error - " + e));
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

loadSettings().catch((e) => setStatus("error - " + e));
showHotkeyStatus().catch(() => {});
