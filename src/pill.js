// The pill's orb comes from the "thinking-orbs" package. Its /engine export is plain
// geometry + canvas code (no React), vendored in ./vendor (refresh: npm run vendor:orbs).
// The package paints in grey only, so we take its frames and paint the dots in yapp's
// purple -> pink ourselves.
import { MODE_FRAMES, resolvePreset } from "./vendor/thinking-orbs-engine.js";

const { listen } = window.__TAURI__.event;

// yapp pill state -> orb state (real state names from thinking-orbs) and label.
const ORB_STATES = {
  recording: { orb: "listening", text: "keep yapping" }, // a waveform rolling through rings
  processing: { orb: "composing", text: "cleaning up your yap…" }, // an undulating multi-band sash
};

const PURPLE = [124, 92, 255];
const PINK = [255, 110, 199];
const ORB_PRESET = 20; // the inline-scale preset: chunkier dots that stay legible at pill size (the other tuned size is 64)

const pill = document.getElementById("pill");
const canvas = document.getElementById("orb");
const label = document.getElementById("label");
const ctx = canvas.getContext("2d");
const reduceMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;

const dpr = Math.min(2, window.devicePixelRatio || 1);
const cssSize = () => parseFloat(getComputedStyle(canvas).width) || 38;
function sizeCanvas() {
  canvas.width = canvas.height = Math.round(cssSize() * dpr);
}

let mode = null; // { frame, opts, speed, kind }
let t = 0; // orb clock in seconds
let level = 0; // smoothed mic loudness 0..1
let targetLevel = 0;
let lastTick = 0;
let raf = 0;

listen("yapp://level", (e) => {
  targetLevel = Math.min(1, Math.max(0, Number(e.payload) || 0));
});

const mix = (a, b, f) => a + (b - a) * f;

function paint(frame, k) {
  const px = (cssSize() * dpr) / ORB_PRESET; // frame units -> device pixels
  const c = ORB_PRESET / 2;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  // Scale about the centre so the whole orb can swell with the voice.
  ctx.setTransform(px * k, 0, 0, px * k, (canvas.width / 2) - c * px * k, (canvas.height / 2) - c * px * k);
  for (const d of frame.dots) {
    const near = 1 - Math.min(1, Math.max(0, d.white)); // the pill is dark: near dots read bright
    const f = Math.min(1, Math.max(0, d.x / ORB_PRESET));
    const lift = 0.05 + 0.3 * near; // pull near dots toward white so depth still reads
    const r = Math.round(mix(mix(PURPLE[0], PINK[0], f), 255, lift));
    const g = Math.round(mix(mix(PURPLE[1], PINK[1], f), 255, lift));
    const b = Math.round(mix(mix(PURPLE[2], PINK[2], f), 255, lift));
    ctx.fillStyle = `rgba(${r},${g},${b},${(d.a ?? 1) * (0.55 + 0.45 * near)})`;
    ctx.beginPath();
    ctx.arc(d.x, d.y, d.r, 0, Math.PI * 2);
    ctx.fill();
  }
}

function tick(now) {
  const dt = Math.min(0.05, (now - lastTick) / 1000 || 0.016);
  lastTick = now;
  let speedMul = 1;
  let k = 1;
  if (mode.kind === "recording") {
    targetLevel *= 0.9; // fade if level events pause
    level += (targetLevel - level) * 0.3;
    speedMul = 0.6 + 2.0 * level; // faster when you talk louder
    k = 0.88 + 0.24 * level; // and the orb swells with your voice
  }
  t += dt * mode.speed * speedMul;
  paint(mode.frame(ORB_PRESET, t, mode.opts), k);
  raf = requestAnimationFrame(tick);
}

function startOrb(kind) {
  const { orb } = ORB_STATES[kind];
  const preset = resolvePreset(orb, ORB_PRESET);
  mode = { kind, frame: MODE_FRAMES[preset.mode], opts: preset.opts, speed: preset.speed };
  level = targetLevel = 0;
  sizeCanvas();
  cancelAnimationFrame(raf);
  if (reduceMotion) {
    paint(mode.frame(ORB_PRESET, 0.6, mode.opts), 1);
    return;
  }
  lastTick = performance.now();
  raf = requestAnimationFrame(tick);
}

function stopOrb() {
  cancelAnimationFrame(raf);
  raf = 0;
}

// The window is shown/hidden by Rust; the slide happens here. `.in` = risen into view.
listen("yapp://pill-hide", () => {
  pill.classList.remove("in");
  setTimeout(() => { if (!pill.classList.contains("in")) stopOrb(); }, 400); // keep animating while it slides away
});

listen("yapp://state", ({ payload: { state, message, short } }) => {
  pill.classList.remove("idle", "recording", "processing", "error");
  pill.classList.add(state);
  if (state !== "idle") pill.classList.add("in");
  if (state === "recording" || state === "processing") {
    label.textContent = ORB_STATES[state].text;
    if (!raf || mode?.kind !== state) startOrb(state); // repeated "processing" events must not restart the animation
  } else if (state === "error") {
    stopOrb();
    label.replaceChildren();
    const title = document.createElement("b");
    title.textContent = "Failed";
    const why = document.createElement("span");
    why.textContent = short || message;
    label.append(title, why);
  }
  // "idle": nothing to do here - Rust follows up with yapp://pill-hide, and the orb
  // keeps moving while the pill slides out.
});
