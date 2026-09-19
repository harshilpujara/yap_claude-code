const { listen } = window.__TAURI__.event;

const BARS = 28;
const pill = document.getElementById("pill");
const wave = document.getElementById("wave");
const label = document.getElementById("label");
const bars = [];
const history = new Array(BARS).fill(0);

for (let i = 0; i < BARS; i++) {
  const b = document.createElement("i");
  b.style.setProperty("--i", i);
  wave.appendChild(b);
  bars.push(b);
}

function draw() {
  bars.forEach((b, i) => {
    b.style.height = 3 + history[i] * 25 + "px";
  });
}

// Newest loudness enters on the right and scrolls left.
listen("yapp://level", (e) => {
  history.shift();
  history.push(Math.min(1, e.payload));
  draw();
});

const WORDS = { processing: "Working" };

listen("yapp://state", ({ payload: { state, message, short } }) => {
  pill.className = "pill " + state;
  if (state === "recording") {
    history.fill(0);
    draw();
    label.textContent = "";
  } else if (state === "processing") {
    label.textContent = WORDS.processing;
    bars.forEach((b) => (b.style.height = ""));
  } else if (state === "error") {
    label.replaceChildren();
    const title = document.createElement("b");
    title.textContent = "Failed";
    const why = document.createElement("span");
    why.textContent = short || message;
    label.append(title, why);
  }
});
draw();
