const { invoke } = window.__TAURI__.core;

const recordBtn = document.getElementById("record");
const statusEl = document.getElementById("status");
const resultEl = document.getElementById("result");
const infoEl = document.getElementById("info");
const transcriptEl = document.getElementById("transcript");
const revealBtn = document.getElementById("reveal");
const settingsEl = document.getElementById("settings");
const baseUrlEl = document.getElementById("base-url");
const modelEl = document.getElementById("model");
const apiKeyEl = document.getElementById("api-key");
const keyNoteEl = document.getElementById("key-note");
const saveBtn = document.getElementById("save");
const clearKeyBtn = document.getElementById("clear-key");

let recording = false;
let lastPath = null;

function setStatus(text) {
  statusEl.textContent = "Status: " + text;
}

async function loadSettings() {
  const s = await invoke("get_stt_settings");
  baseUrlEl.value = s.base_url;
  modelEl.value = s.model;
  apiKeyEl.value = "";
  keyNoteEl.textContent = s.has_key
    ? "A key is saved. Leave the box empty to keep it."
    : "No key saved yet. Paste your Groq API key above.";
  if (!s.has_key) settingsEl.open = true;
  return s;
}

async function saveSettings(apiKey) {
  try {
    await invoke("save_stt_settings", {
      baseUrl: baseUrlEl.value,
      model: modelEl.value,
      apiKey,
    });
    await loadSettings();
    setStatus("settings saved");
  } catch (e) {
    setStatus("error - " + e);
  }
}

saveBtn.addEventListener("click", () => {
  const k = apiKeyEl.value.trim();
  saveSettings(k === "" ? null : k);
});
clearKeyBtn.addEventListener("click", () => saveSettings(""));

async function start() {
  if (recording) return;
  recording = true;
  recordBtn.classList.add("active");
  recordBtn.textContent = "Recording... release to stop";
  setStatus("recording");
  try {
    await invoke("start_recording");
  } catch (e) {
    recording = false;
    recordBtn.classList.remove("active");
    recordBtn.textContent = "Hold to record";
    setStatus("error - " + e);
  }
}

async function stop() {
  if (!recording) return;
  recording = false;
  recordBtn.classList.remove("active");
  recordBtn.textContent = "Hold to record";
  setStatus("saving...");
  let r;
  try {
    r = await invoke("stop_recording");
  } catch (e) {
    setStatus("error - " + e);
    return;
  }
  lastPath = r.path;
  const quiet = r.peak < 0.01 ? " WARNING: almost silent - check your microphone." : "";
  infoEl.textContent = `Recorded ${r.original_seconds.toFixed(1)}s, sent ${r.seconds.toFixed(1)}s after trimming silence.${quiet}`;
  resultEl.hidden = false;
  transcriptEl.textContent = "";

  setStatus("transcribing...");
  const t0 = performance.now();
  try {
    const text = await invoke("transcribe_last");
    const secs = ((performance.now() - t0) / 1000).toFixed(1);
    transcriptEl.textContent = text || "(no speech detected)";
    setStatus(`done (transcribed in ${secs}s)`);
  } catch (e) {
    setStatus("error - " + e);
  }
}

recordBtn.addEventListener("pointerdown", (e) => {
  recordBtn.setPointerCapture(e.pointerId);
  start();
});
recordBtn.addEventListener("pointerup", stop);
recordBtn.addEventListener("pointercancel", stop);

revealBtn.addEventListener("click", () => {
  if (lastPath) invoke("reveal_recording", { path: lastPath }).catch((e) => setStatus("error - " + e));
});

loadSettings().catch((e) => setStatus("error - " + e));
