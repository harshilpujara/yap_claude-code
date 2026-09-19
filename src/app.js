const { invoke } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);

const recordBtn = $("record");
const statusEl = $("status");
let recording = false;
let lastPath = null;

function setStatus(text) {
  statusEl.textContent = "Status: " + text;
}

// ---------- Settings ----------
async function loadSettings() {
  const s = await invoke("get_settings");
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
  if (!s.has_stt_key) $("settings").open = true;
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
      },
      sttKey,
      llmKey,
    });
    await loadSettings();
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

// ---------- Recording pipeline ----------
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
  $("info").textContent = `Recorded ${r.original_seconds.toFixed(1)}s, sent ${r.seconds.toFixed(1)}s after trimming silence.${quiet}`;
  $("result").hidden = false;
  $("raw").textContent = "";
  $("clean").textContent = "";

  setStatus("transcribing...");
  const t0 = performance.now();
  let raw;
  try {
    raw = await invoke("transcribe_last");
  } catch (e) {
    setStatus("error - " + e);
    return;
  }
  const sttSecs = ((performance.now() - t0) / 1000).toFixed(1);
  $("raw").textContent = raw || "(no speech detected)";
  if (!raw) {
    setStatus(`done (transcribed in ${sttSecs}s, nothing to clean)`);
    return;
  }

  setStatus("cleaning up...");
  const t1 = performance.now();
  try {
    const clean = await invoke("cleanup_text", { raw });
    const llmSecs = ((performance.now() - t1) / 1000).toFixed(1);
    $("clean").textContent = clean || "(empty result)";
    setStatus(`done (transcribed ${sttSecs}s + cleaned ${llmSecs}s)`);
  } catch (e) {
    setStatus("cleanup error - " + e);
  }
}

recordBtn.addEventListener("pointerdown", (e) => {
  recordBtn.setPointerCapture(e.pointerId);
  start();
});
recordBtn.addEventListener("pointerup", stop);
recordBtn.addEventListener("pointercancel", stop);

$("reveal").addEventListener("click", () => {
  if (lastPath) invoke("reveal_recording", { path: lastPath }).catch((e) => setStatus("error - " + e));
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
