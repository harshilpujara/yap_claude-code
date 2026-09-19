const { invoke } = window.__TAURI__.core;

const recordBtn = document.getElementById("record");
const statusEl = document.getElementById("status");
const resultEl = document.getElementById("result");
const infoEl = document.getElementById("info");
const revealBtn = document.getElementById("reveal");

let recording = false;
let lastPath = null;

function setStatus(text) {
  statusEl.textContent = "Status: " + text;
}

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
  try {
    const r = await invoke("stop_recording");
    lastPath = r.path;
    const quiet = r.peak < 0.01 ? " WARNING: almost silent - check your microphone." : "";
    infoEl.textContent = `Saved ${r.seconds.toFixed(1)}s to ${r.path}.${quiet}`;
    resultEl.hidden = false;
    setStatus("saved");
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
