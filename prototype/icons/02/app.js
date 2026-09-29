import { DotLottie } from "../../vendor/dotlottie-web.js";
DotLottie.setWasmUrl(
  new URL("../../vendor/dotlottie-player.wasm", import.meta.url).href,
);

const reduced = matchMedia("(prefers-reduced-motion: reduce)");
const loop = document.querySelector("#loop");
const replayAll = document.querySelector("#replay-all");
const speed = document.querySelector("#speed");
const note = document.querySelector("#motion-note");
const cards = [];
const weightControl = document.querySelector("#stroke-width");
let iconStyle = "outline", strokeWidth = 2;
let theme = "dark",
  swatch = "teal",
  pageActive = true;
const palette = {
  dark: { teal: "#00DDB3", aqua: "#59E4E8", mint: "#B7F5E6" },
  light: { teal: "#008C76", aqua: "#087D87", mint: "#247866" },
};
const rgba = (hex) =>
  [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255).concat(1);

function load(canvas, data) {
  const player = new DotLottie({
    canvas,
    data,
    autoplay: false,
    loop: false,
    useFrameInterpolation: true,
    renderConfig: {
      devicePixelRatio: Math.min(devicePixelRatio || 1, 2),
      autoResize: true,
      freezeOnOffscreen: false,
    },
  });
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      player.destroy();
      reject(new Error("Lottie load timeout"));
    }, 20000);
    player.addEventListener("load", () => {
      clearTimeout(timer);
      resolve(player);
    });
    player.addEventListener("loadError", () => {
      clearTimeout(timer);
      player.destroy();
      reject(new Error("Lottie could not load"));
    });
  });
}
function bindStyle(player) {
  for (const [name, width] of [["light", 1.5], ["regular", 2], ["bold", 2.5]])
    player.setScalarSlot(`style.outline.${name}`, iconStyle !== "solid" && strokeWidth === width ? 100 : 0);
  player.setScalarSlot("style.solid", iconStyle === "solid" ? 100 : 0);
  player.setScalarSlot("style.tint", iconStyle === "duotone" ? 18 : 0);
}
function applyStyle() {
  for (const button of document.querySelectorAll("button[data-style]"))
    button.setAttribute("aria-pressed", String(button.dataset.style === iconStyle));
  weightControl.disabled = iconStyle === "solid";
  weightControl.value = String(strokeWidth);
  weightControl.style.setProperty("--weight-progress", `${(strokeWidth - 1.5) * 100}%`);
  const weightName = { 1.5: "Light", 2: "Regular", 2.5: "Bold" }[strokeWidth];
  weightControl.setAttribute("aria-valuetext", `${weightName}, ${strokeWidth.toFixed(1)} units`);
  document.querySelector("#weight-value").value = iconStyle === "solid" ? "—" : strokeWidth.toFixed(1);
  document.querySelector("#weight-detail").textContent = {
    outline: "Open contours. Round terminals. Weight in a 32-unit grid.",
    solid: "A separate filled master. Balanced mass and clear cutouts. No stroke weight.",
    duotone: "A strong outline with an 18% tint. The same motion, a little more presence.",
  }[iconStyle];
  for (const card of cards) for (const player of card.players) {
    const frame = player.currentFrame;
    bindStyle(player);
    player.setFrame(frame);
  }
}
function applyTheme() {
  document.documentElement.dataset.theme = theme;
  const color = palette[theme][swatch];
  document.documentElement.style.setProperty("--accent", color);
  document.querySelector("#color-value").value = color;
  for (const button of document.querySelectorAll("button[data-theme]"))
    if (button.dataset.theme)
      button.setAttribute(
        "aria-pressed",
        String(button.dataset.theme === theme),
      );
  for (const button of document.querySelectorAll("[data-color]")) {
    button.setAttribute(
      "aria-pressed",
      String(button.dataset.color === swatch),
    );
    button.style.setProperty("--swatch", palette[theme][button.dataset.color]);
  }
  for (const card of cards)
    for (const player of card.players) {
      const frame = player.currentFrame;
      player.setTheme(theme);
      player.setColorSlot("color.primary", rgba(color));
      bindStyle(player);
      player.setFrame(frame);
    }
}
function updateAccessibility() {
  replayAll.disabled = reduced.matches || !cards.some((card) => card.ready);
  loop.disabled = reduced.matches;
  speed.disabled = reduced.matches;
  note.textContent = reduced.matches
    ? "Reduced motion is on. The icon shows its resting pose."
    : "Hover, tap, or replay. Use the timeline to look a little closer.";
  for (const card of cards) {
    card.stage.disabled = reduced.matches || !card.ready;
    card.replay.disabled = reduced.matches || !card.ready;
    card.scrub.disabled = reduced.matches || !card.ready;
    if (reduced.matches) {
      card.running = false;
      card.rest();
    }
  }
}
function resume(card) {
  if (!card.ready) return;
  if (
    !card.running ||
    !card.visible ||
    document.hidden ||
    !pageActive ||
    reduced.matches
  ) {
    for (const player of card.players) player.pause();
    return;
  }
  const frame = card.players[0].currentFrame;
  for (const player of card.players) {
    player.setFrame(frame);
    player.play();
  }
}
async function mount(root, icon) {
  let meta = icon.motions.find(motion => motion.motionId === 'light');
  const card = {
    root,
    meta,
    ready: false,
    players: [],
    running: false,
    visible: !("IntersectionObserver" in window),
    stage: root.querySelector(".stage"),
    replay: root.querySelector(".replay-icon"),
    scrub: root.querySelector(".scrub"),
    motionButtons: [...root.querySelectorAll('[data-motion]')],
  };
  const status = root.querySelector(".state"),
    time = root.querySelector(".time");
  const progress = (frame) => {
    card.scrub.value = frame;
    card.scrub.setAttribute(
      "aria-valuetext",
      `${(frame / 60).toFixed(2)} seconds`,
    );
    time.value = `${(frame / 60).toFixed(2)} s`;
  };
  const label = (playing, text) => {
    root.dataset.playing = String(playing);
    status.textContent = text;
  };
  function updateMotionDetails() {
    root.dataset.motion = meta.motionId;
    for (const button of card.motionButtons) {
      button.disabled = !card.ready;
      button.setAttribute('aria-pressed', String(button.dataset.motion === meta.motionId));
    }
    root.querySelector('.card-info h3').textContent = meta.heading;
    root.querySelector('.card-info p').textContent = meta.description;
    const download = root.querySelector('.download-link');
    download.href = `assets/${meta.name}.lottie?v=${meta.sha256}`;
    download.setAttribute('aria-label', `Download ${icon.label}, ${meta.motionLabel.toLowerCase()}, all three styles`);
    root.querySelector('.file-size').textContent = `${(meta.bytes / 1024).toFixed(1)} KB`;
    root.querySelector('.gesture-detail').textContent = `3 styles · ${meta.active.toFixed(2)} s gesture`;
    card.scrub.max = meta.frames - 1;
  }
  card.rest = () => {
    for (const player of card.players) {
      player.pause();
      player.setSegment(0, meta.frames - 1);
      player.setFrame(meta.rest);
    }
    progress(meta.rest);
    label(false, "At rest");
  };
  card.play = (force = false) => {
    if (!card.ready || reduced.matches || (!force && card.running)) return;
    for (const player of card.players) {
      player.pause();
      player.setSegment(
        0,
        loop.checked ? meta.frames - 1 : Math.ceil(meta.active * 60),
      );
      player.setLoop(loop.checked);
      player.setFrame(0);
    }
    card.running = true;
    label(true, "Playing");
    resume(card);
  };
  card.selectMotion = (id) => {
    const next = icon.motions.find(motion => motion.motionId === id);
    if (!card.ready || !next || next === meta) return;
    card.running = false;
    for (const player of card.players) player.pause();
    const previous = meta;
    try {
      meta = next;
      card.meta = meta;
      for (const player of card.players) {
        player.loadAnimation(meta.name);
        if (player.activeAnimationId !== meta.name) throw new Error('Motion could not load');
        player.setSpeed(Number(speed.value));
        player.setTheme(theme);
        player.setColorSlot('color.primary', rgba(palette[theme][swatch]));
        bindStyle(player);
      }
      updateMotionDetails();
      card.rest();
      card.play(true);
    } catch (error) {
      meta = previous;
      card.meta = meta;
      for (const player of card.players) player.loadAnimation(meta.name);
      applyTheme();
      updateMotionDetails();
      card.rest();
      status.textContent = 'Motion unavailable';
      console.error(error);
    }
  };
  cards.push(card);
  try {
    const response = await fetch(
      new URL(`./assets/${meta.name}.lottie?v=${meta.sha256}`, import.meta.url),
      { cache: "no-store" },
    );
    if (!response.ok)
      throw new Error(`Lottie request failed: ${response.status}`);
    const data = await response.arrayBuffer();
    const results = await Promise.allSettled(
      [...root.querySelectorAll("canvas")].map((canvas) => load(canvas, data)),
    );
    card.players = results
      .filter((result) => result.status === "fulfilled")
      .map((result) => result.value);
    if (results.some((result) => result.status === "rejected"))
      throw new Error("A preview player could not load");
    card.ready = true;
    updateMotionDetails();
    for (const button of card.motionButtons)
      button.addEventListener('click', () => card.selectMotion(button.dataset.motion));
    card.players[0].addEventListener("frame", (event) =>
      progress(event.currentFrame),
    );
    card.players[0].addEventListener("complete", () => {
      if (!loop.checked) {
        card.running = false;
        card.rest();
      }
    });
    card.stage.addEventListener("pointerenter", () => card.play());
    card.stage.addEventListener("focus", () => card.play());
    card.stage.addEventListener("click", () => card.play());
    card.replay.addEventListener("click", () => card.play(true));
    card.scrub.addEventListener("input", () => {
      card.running = false;
      for (const player of card.players) {
        player.pause();
        player.setSegment(0, meta.frames - 1);
        player.setFrame(Number(card.scrub.value));
      }
      progress(Number(card.scrub.value));
      label(false, "Paused");
    });
    if ("IntersectionObserver" in window) {
      card.observer = new IntersectionObserver(
        (entries) => {
          card.visible = entries[0].isIntersecting;
          if (card.visible && !card.started) {
            card.started = true;
            card.play();
          }
          resume(card);
        },
        { threshold: 0.2 },
      );
      card.observer.observe(card.stage);
    }
    applyTheme();
    card.rest();
    updateAccessibility();
    if (card.visible) card.play();
  } catch (error) {
    for (const player of card.players) player.destroy();
    card.players = [];
    card.ready = false;
    root.classList.add("load-error");
    label(false, "Unavailable");
    console.error(error);
    updateAccessibility();
  }
}

document.querySelectorAll("button[data-style]").forEach((button) =>
  button.addEventListener("click", () => {
    iconStyle = button.dataset.style;
    applyStyle();
  }),
);
weightControl.addEventListener("input", () => {
  strokeWidth = Math.round(Math.max(1.5, Math.min(2.5, Number(weightControl.value) || 2)) * 2) / 2;
  applyStyle();
});
document.querySelectorAll("button[data-theme]").forEach((button) =>
  button.addEventListener("click", () => {
    theme = button.dataset.theme;
    applyTheme();
  }),
);
document.querySelectorAll("[data-color]").forEach((button) =>
  button.addEventListener("click", () => {
    swatch = button.dataset.color;
    applyTheme();
  }),
);
replayAll.addEventListener("click", () =>
  cards.forEach((card) => card.play(true)),
);
speed.addEventListener("change", () => {
  for (const card of cards)
    for (const player of card.players) player.setSpeed(Number(speed.value));
});
loop.addEventListener("change", () => {
  for (const card of cards) {
    if (loop.checked) card.play(true);
    else {
      card.running = false;
      card.rest();
    }
  }
});
reduced.addEventListener("change", updateAccessibility);
document.addEventListener("visibilitychange", () => cards.forEach(resume));
window.addEventListener("pagehide", (event) => {
  pageActive = false;
  if (event.persisted) cards.forEach(resume);
  else
    for (const card of cards) {
      card.observer?.disconnect();
      card.players.forEach((player) => player.destroy());
    }
});
window.addEventListener("pageshow", () => {
  pageActive = true;
  cards.forEach(resume);
});
try {
  const response = await fetch(new URL("./assets/set.json?v=3", import.meta.url), { cache: "no-store" });
  if (!response.ok) throw new Error("Set manifest could not load");
  const set = await response.json();
  await Promise.all(
    set.icons.map((meta) =>
      mount(document.querySelector(`[data-icon="${meta.name}"]`), meta),
    ),
  );
} catch (error) {
  document.querySelectorAll(".icon-card").forEach((card) => {
    card.classList.add("load-error");
    card.querySelector(".state").textContent = "Unavailable";
  });
  note.textContent = "The preview could not load. Reload to try again.";
  console.error(error);
}
updateAccessibility();
