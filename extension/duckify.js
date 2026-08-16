(function duckify() {
  const PORT = 8787;

  if (!Spicetify?.Player || !Spicetify?.React || !Spicetify?.ReactDOM) {
    setTimeout(duckify, 300);
    return;
  }

  const { React } = Spicetify;

  // styles
  const CSS = `
.dkf {
  --dkf-accent: var(--spice-button-active, #1ed760);
  --dkf-text: var(--spice-text, #fff);
  --dkf-subtext: var(--spice-subtext, #b3b3b3);
  --dkf-track: rgba(var(--spice-rgb-selected-row, 255,255,255), .18);
  font-family: var(--encore-body-font-stack, CircularSp, ui-sans-serif, sans-serif);
  color: var(--dkf-text);
  display: flex;
  flex-direction: column;
  gap: 26px;
  width: 100%;
  box-sizing: border-box;
  padding: 16px 28px 28px;
  max-height: min(68vh, 600px);
  overflow-y: auto;
}
.dkf::-webkit-scrollbar { width: 8px; }
.dkf::-webkit-scrollbar-thumb { background: var(--dkf-track); border-radius: 4px; }
.dkf::-webkit-scrollbar-track { background: transparent; }
.dkf-root { background: transparent; }
.dkf-state {
  font-size: 13px;
  color: var(--dkf-subtext);
  padding-bottom: 20px;
  border-bottom: 1px solid var(--dkf-track);
}
.dkf-state b { color: var(--dkf-text); font-weight: 700; }
.dkf-state-row {
  display: flex; align-items: center; justify-content: space-between;
  gap: 16px;
}
.dkf-group { display: flex; flex-direction: column; gap: 20px; }
.dkf-group-label {
  font-size: 11px; font-weight: 700; letter-spacing: .1em;
  text-transform: uppercase; color: var(--dkf-subtext);
}
.dkf-field { display: flex; flex-direction: column; gap: 8px; }
.dkf-field-head {
  display: flex; justify-content: space-between; align-items: baseline; gap: 16px;
}
.dkf-field-label { font-size: 14px; font-weight: 500; }
.dkf-field-value {
  font-size: 13px; font-weight: 700; color: var(--dkf-accent);
  font-variant-numeric: tabular-nums; flex-shrink: 0;
}
.dkf-field-hint { font-size: 12px; color: var(--dkf-subtext); line-height: 1.45; }
.dkf-range {
  -webkit-appearance: none; appearance: none;
  width: 100%; height: 12px; background: transparent; cursor: pointer; margin: 0;
}
.dkf-range::-webkit-slider-runnable-track {
  height: 4px; border-radius: 2px;
  background: linear-gradient(to right,
    var(--dkf-accent) 0%, var(--dkf-accent) var(--dkf-pct,0%),
    var(--dkf-track) var(--dkf-pct,0%), var(--dkf-track) 100%);
}
.dkf-range::-webkit-slider-thumb {
  -webkit-appearance: none; appearance: none;
  width: 12px; height: 12px; border-radius: 50%;
  background: var(--dkf-text); margin-top: -4px;
  opacity: 0; transition: opacity .12s ease;
}
.dkf-range:hover::-webkit-slider-thumb,
.dkf-range:focus-visible::-webkit-slider-thumb { opacity: 1; }
.dkf-range:focus-visible { outline: 2px solid var(--dkf-accent); outline-offset: 4px; }
.dkf-ask { display: flex; flex-direction: column; gap: 18px; }
.dkf-ask-item {
  display: flex; flex-direction: column; gap: 10px;
  padding: 14px; border-radius: 8px;
  background: rgba(var(--spice-rgb-selected-row, 255,255,255), .04);
}
.dkf-ask-q { font-size: 14px; line-height: 1.4; }
.dkf-ask-row { display: flex; align-items: center; gap: 10px; }
.dkf-ask-name {
  font-family: ui-monospace, Consolas, monospace;
  font-size: 13px; color: var(--dkf-accent);
}
.dkf-meter { display: flex; align-items: center; gap: 10px; }
.dkf-meter-track {
  flex: 1; height: 4px; border-radius: 2px;
  background: var(--dkf-track); overflow: hidden;
}
.dkf-meter-fill {
  height: 100%; border-radius: 2px;
  background: var(--dkf-accent);
  transition: width .08s linear;
}
.dkf-meter-label {
  font-size: 11px; color: var(--dkf-subtext);
  font-variant-numeric: tabular-nums;
  min-width: 76px; text-align: right;
}
.dkf-reset {
  background: none; border: 0; padding: 0; cursor: pointer;
  font: inherit; font-size: 12px; color: var(--dkf-subtext);
  text-decoration: underline;
}
.dkf-reset:hover { color: var(--dkf-text); }
.dkf-confirm { gap: 16px; max-height: none; }
.dkf-confirm-text { margin: 0; font-size: 14px; line-height: 1.5; }
.dkf-btn {
  font-family: inherit; font-size: 12px; font-weight: 700;
  padding: 8px 16px; border-radius: 500px; border: none; cursor: pointer;
  transition: transform .1s ease; white-space: nowrap;
}
.dkf-btn:hover { transform: scale(1.04); }
.dkf-btn:active { transform: scale(.98); }
.dkf-btn:focus-visible { outline: 2px solid var(--dkf-text); outline-offset: 2px; }
.dkf-btn-yes { background: var(--dkf-accent); color: #000; }
.dkf-btn-no {
  background: transparent; color: var(--dkf-subtext); border: 1px solid var(--dkf-track);
}
.dkf-btn-no:hover { color: var(--dkf-text); }
.dkf-footer {
  display: flex; justify-content: space-between; align-items: center;
  gap: 16px; flex-wrap: wrap;
  padding-top: 20px; border-top: 1px solid var(--dkf-track);
  font-size: 12px; color: var(--dkf-subtext);
}
.dkf-toggle { display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none; }
.dkf-toggle input { accent-color: var(--dkf-accent); cursor: pointer; }
.dkf-topbar-btn { --dkf-icon-hole: var(--spice-main, #121212); }
.dkf-topbar-btn svg { display: block; }
.dkf[data-connected="false"] .dkf-group,
.dkf[data-connected="false"] .dkf-footer { opacity: .4; pointer-events: none; }
.main-trackCreditsModal-header {
  display: flex; align-items: center; justify-content: space-between;
  gap: 16px; padding: 24px 28px 4px;
}
.main-trackCreditsModal-header h1 {
  margin: 0; font-size: 26px; font-weight: 700; line-height: 1.2;
}
.main-trackCreditsModal-closeBtn {
  width: 32px; height: 32px; border-radius: 32px; flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  background: transparent; border: 0; cursor: pointer;
  color: #ffffffb3; padding: 0;
}
.main-trackCreditsModal-closeBtn:hover { background: #ffffff1a; color: #fff; }
.main-trackCreditsModal-closeBtn svg { fill: currentColor; width: 16px; height: 16px; }
`;

  function injectStyle() {
    if (document.getElementById("dkf-style")) return;
    const el = document.createElement("style");
    el.id = "dkf-style";
    el.textContent = CSS;
    document.head.appendChild(el);
  }

  // state
  const state = {
    connected: false,
    reason: "idle",
    detail: "",
    candidates: [],
    knownGames: 0,
    config: null,
    holdUntil: 0,
    baseline: 1,
    active: 1,
    held: false,
    pausing: false,
    resuming: false,
    override: false,
    offering: false,
    settleUntil: 0,
    starting: false,
    startFailed: false,
    autostart: false,
  };

  let socket = null;
  let fadeTimer = null;
  let pushTimer = null;
  let lastSet = null;
  let reconnectDelay = 1000;
  const listeners = new Set();
  const announced = new Set();
  const notify = () => listeners.forEach((fn) => fn());

  // volume

  function setVolume(v) {
    lastSet = Math.max(0, Math.min(1, v));
    Spicetify.Player.setVolume(lastSet);
  }

  function syncBaseline() {
    if (fadeTimer || state.held || state.pausing || state.resuming) return;

    const now = Spicetify.Player.getVolume();
    if (lastSet !== null && Math.abs(now - lastSet) <= 0.02) return;

    const factor = state.active || 1;
    state.baseline = factor > 0.01 ? Math.min(1, now / factor) : now;
    lastSet = now;
  }

  function rampVolume(target, ms, done) {
    target = Math.max(0, Math.min(1, target));

    if (!fadeTimer && lastSet !== null && Math.abs(target - lastSet) < 0.005) {
      done?.();
      return;
    }

    clearInterval(fadeTimer);
    fadeTimer = null;

    const start = Spicetify.Player.getVolume();
    const delta = target - start;

    if (Math.abs(delta) < 0.005 || ms <= 0) {
      setVolume(target);
      done?.();
      return;
    }

    const steps = Math.max(1, Math.round(ms / 16));
    let i = 0;
    fadeTimer = setInterval(() => {
      i += 1;
      const t = i / steps;
      setVolume(start + delta * (1 - Math.pow(1 - t, 2)));
      if (i >= steps) {
        clearInterval(fadeTimer);
        fadeTimer = null;
        setVolume(target);
        done?.();
      }
    }, 16);
  }

  function apply(msg) {
    state.reason = msg.reason;
    state.detail = msg.detail;
    state.candidates = msg.candidates ?? [];
    state.knownGames = msg.known_games ?? 0;
    state.autostart = !!msg.autostart;
    if (msg.config && Date.now() > state.holdUntil) state.config = msg.config;

    announce(state.candidates);

    syncBaseline();

    const playing = Spicetify.Player.isPlaying();

    if (!msg.pause) {
      state.override = false;
      state.pausing = false;
      state.active = msg.volume;
      const target = state.baseline * msg.volume;

      if (state.held && !state.resuming) {
        state.resuming = true;
        state.settleUntil = Date.now() + 700;
        setVolume(0);
        Spicetify.Player.play();

        let waited = 0;
        const poll = setInterval(() => {
          waited += 40;
          if (Spicetify.Player.isPlaying()) {
            clearInterval(poll);
            state.held = false;
            state.resuming = false;
            state.settleUntil = Date.now() + 700;
            setVolume(0);
            rampVolume(target, msg.fade_ms);
          } else if (waited >= 2000) {
            clearInterval(poll);
            state.resuming = false;
          }
        }, 40);
      } else if (!state.held) {
        rampVolume(target, msg.fade_ms);
      }
      notify();
      return;
    }

    if (
      state.held &&
      playing &&
      !state.resuming &&
      Date.now() > state.settleUntil
    ) {
      state.override = true;
      state.held = false;
      offerOverride(msg.detail);
    }

    if (state.override) {
      notify();
      return;
    }

    if (playing && !state.pausing) {
      state.pausing = true;
      rampVolume(0, msg.fade_ms, () => {
        if (state.override) {
          state.pausing = false;
          return;
        }
        Spicetify.Player.pause();
        state.held = true;
        state.pausing = false;
        state.settleUntil = Date.now() + 700;
      });
    }

    notify();
  }

  // notify
  function offerOverride(detail) {
    if (state.offering) return;
    state.offering = true;

    const watch = setInterval(() => {
      if (!document.querySelector(".dkf-confirm")) {
        clearInterval(watch);
        state.offering = false;
      }
    }, 400);

    const source = (detail || "").split(" ")[0] || "that app";
    const box = document.createElement("div");
    box.className = "dkf-root";

    Spicetify.ReactDOM.render(
      h(
        "div",
        { className: "dkf dkf-confirm" },
        h(
          "p",
          { className: "dkf-confirm-text" },
          "Duckify paused your music because ",
          h("span", { className: "dkf-ask-name" }, source),
          " was making sound, and you started it again.",
        ),
        h(
          "p",
          { className: "dkf-confirm-text" },
          "Keep playing until it goes quiet, or stop pausing for this every time?",
        ),
        h(
          "div",
          { className: "dkf-ask-row" },
          h(
            "button",
            {
              className: "dkf-btn dkf-btn-yes",
              onClick: () => {
                send({ type: "classify", process: source, is_game: false });
                state.offering = false;
                Spicetify.PopupModal.hide();
              },
            },
            "Never pause for this",
          ),
          h(
            "button",
            {
              className: "dkf-btn dkf-btn-no",
              onClick: () => {
                state.offering = false;
                Spicetify.PopupModal.hide();
              },
            },
            "Just this once",
          ),
        ),
      ),
      box,
    );

    Spicetify.PopupModal.display({ title: "Keep playing?", content: box });
  }

  function announce(candidates) {
    const names = candidates.map((c) => c.process);
    for (const proc of names) {
      if (announced.has(proc)) continue;
      announced.add(proc);
      try {
        Spicetify.Snackbar.enqueueSnackbar(
          `${proc} is making sound. Click to decide if it should quiet your music.`,
          { variant: "default", autoHideDuration: 8000 },
        );
        makeClickable();
      } catch {}
    }
    for (const proc of [...announced]) {
      if (!names.includes(proc)) announced.delete(proc);
    }
  }

  // Snackbar takes only text, so the click target is wired up on the element
  // once it renders.
  function makeClickable() {
    let tries = 0;
    const find = setInterval(() => {
      tries += 1;
      const nodes = document.querySelectorAll(
        ".encore-announcement-set, [class*='Snackbar'], [class*='snackbar']",
      );
      for (const el of nodes) {
        if (el.dataset.dkfBound) continue;
        if (!/quiet your music/i.test(el.textContent || "")) continue;
        el.dataset.dkfBound = "1";
        el.style.cursor = "pointer";
        el.addEventListener("click", () => {
          openPanel();
          el.remove();
        });
      }
      if (tries > 12) clearInterval(find);
    }, 150);
  }

  // socket
  function connect() {
    // Guard against overlapping attempts leaving orphaned sockets behind.
    if (
      socket &&
      (socket.readyState === WebSocket.OPEN ||
        socket.readyState === WebSocket.CONNECTING)
    ) {
      return;
    }
    try {
      socket = new WebSocket(`ws://127.0.0.1:${PORT}`);
    } catch {
      scheduleReconnect();
      return;
    }

    socket.onopen = () => {
      state.connected = true;
      reconnectDelay = 1000;
      send({ type: "hello" });
      notify();
    };

    socket.onmessage = (ev) => {
      let msg;
      try {
        msg = JSON.parse(ev.data);
      } catch {
        return;
      }
      if (msg.type === "state") apply(msg);
    };

    socket.onclose = () => {
      state.connected = false;
      notify();
      scheduleReconnect();
    };

    socket.onerror = () => socket?.close();
  }

  function scheduleReconnect() {
    setTimeout(connect, reconnectDelay);
    reconnectDelay = Math.min(reconnectDelay * 1.5, 15000);
  }

  function startHelper() {
    if (state.starting) return;
    state.starting = true;
    state.startFailed = false;
    notify();

    // Electron blocks the renderer from launching a process or navigating to an
    // external protocol, so the page cannot start the helper itself. Windows
    // does honour the protocol from a shell, so this both tries it and keeps
    // reconnecting in case the user starts it another way.
    try {
      window.open("duckify://start", "_blank");
    } catch {}

    reconnectDelay = 1000;
    let waited = 0;
    const poll = setInterval(() => {
      waited += 500;
      if (state.connected) {
        clearInterval(poll);
        state.starting = false;
        notify();
      } else if (waited >= 5000) {
        clearInterval(poll);
        state.starting = false;
        state.startFailed = true;
        notify();
      } else {
        connect();
      }
    }, 500);
  }

  function send(obj) {
    if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(obj));
  }

  // ui
  const h = (tag, props, ...kids) =>
    React.createElement(tag, props, ...kids.filter((k) => k !== false && k != null));

  const STATE_TEXT = {
    idle: "Normal",
    "game-silent": "Playing quietly",
    "game-audible": "Paused for game audio",
    "voice-active": "Ducked for voice",
    disabled: "Turned off",
  };

  function Slider({ label, hint, value, onChange, min, max, step, format }) {
    const pct = ((value - min) / (max - min)) * 100;
    return h(
      "div",
      { className: "dkf-field" },
      h(
        "div",
        { className: "dkf-field-head" },
        h("span", { className: "dkf-field-label" }, label),
        h("span", { className: "dkf-field-value" }, format(value)),
      ),
      h("input", {
        type: "range",
        className: "dkf-range",
        style: { "--dkf-pct": `${pct}%` },
        min,
        max,
        step,
        value,
        "aria-label": label,
        onChange: (e) => onChange(parseFloat(e.target.value)),
      }),
      hint && h("div", { className: "dkf-field-hint" }, hint),
    );
  }

  function Group({ label, children }) {
    return h(
      "div",
      { className: "dkf-group" },
      h("div", { className: "dkf-group-label" }, label),
      ...(Array.isArray(children) ? children : [children]),
    );
  }

  function Settings() {
    const [, force] = React.useReducer((n) => n + 1, 0);

    React.useEffect(() => {
      listeners.add(force);
      return () => listeners.delete(force);
    }, []);

    const cfg = state.config ?? {};

    const update = (patch) => {
      state.config = { ...cfg, ...patch };
      state.holdUntil = Date.now() + 2000;
      notify();

      clearTimeout(pushTimer);
      pushTimer = setTimeout(() => {
        send({ type: "config", config: state.config });
      }, 120);
    };

    const pct = (v) => `${Math.round(v * 100)}%`;
    const ms = (v) => (v >= 1000 ? `${(v / 1000).toFixed(1)}s` : `${v} ms`);

    return h(
      "div",
      { className: "dkf", "data-connected": String(state.connected) },

      h(
        "div",
        { className: "dkf-state" },
        h(
          "div",
          { className: "dkf-state-row" },
          h(
            "span",
            null,
            h(
              "b",
              null,
              state.connected
                ? (STATE_TEXT[state.reason] ?? "Normal")
                : "Helper not running",
            ),
            state.connected && state.detail ? `: ${state.detail}` : "",
          ),
          !state.connected &&
            h(
              "button",
              {
                className: "dkf-btn dkf-btn-yes",
                onClick: startHelper,
              },
              state.starting ? "Starting…" : "Start helper",
            ),
        ),
        !state.connected &&
          state.startFailed &&
          h(
            "div",
            { className: "dkf-field-hint", style: { marginTop: "8px" } },
            "Spotify is not allowed to launch programs, so this could not start it. Run duckify-helper.exe yourself, or turn on Start with Windows so it is always running.",
          ),
      ),

      state.candidates.length > 0 &&
        h(
          Group,
          { label: "Unrecognised apps" },
          h(
            "div",
            { className: "dkf-ask" },
            h(
              "div",
              { className: "dkf-field-hint" },
              "Duckify ignores these until you decide. They stay listed until answered, even after they go quiet.",
            ),
            ...state.candidates.map((c) =>
              h(
                "div",
                { className: "dkf-ask-item", key: c.process },
                h(
                  "div",
                  { className: "dkf-ask-q" },
                  "Should ",
                  h("span", { className: "dkf-ask-name" }, c.process),
                  " quiet your music?",
                ),
                h(
                  "div",
                  { className: "dkf-meter" },
                  h(
                    "div",
                    { className: "dkf-meter-track" },
                    h("div", {
                      className: "dkf-meter-fill",
                      style: { width: `${Math.min(100, c.peak * 140)}%` },
                    }),
                  ),
                  h(
                    "span",
                    { className: "dkf-meter-label" },
                    c.active ? `${Math.round(c.peak * 100)}% now` : "silent",
                  ),
                ),
                h(
                  "div",
                  { className: "dkf-ask-row" },
                  h(
                    "button",
                    {
                      className: "dkf-btn dkf-btn-yes",
                      onClick: () =>
                        send({ type: "classify", process: c.process, is_game: true }),
                    },
                    "Yes",
                  ),
                  h(
                    "button",
                    {
                      className: "dkf-btn dkf-btn-no",
                      onClick: () =>
                        send({ type: "classify", process: c.process, is_game: false }),
                    },
                    "No",
                  ),
                ),
              ),
            ),
          ),
        ),

      h(
        Group,
        { label: "Games" },
        h(Slider, {
          key: "gsv",
          label: "Volume while a game is quiet",
          hint: "Music keeps playing softly instead of stopping.",
          value: cfg.game_silent_volume ?? 0.1,
          min: 0,
          max: 1,
          step: 0.01,
          format: pct,
          onChange: (v) => update({ game_silent_volume: v }),
        }),
        h(Slider, {
          key: "thr",
          label: "How loud counts as sound",
          hint: "Raise this if quiet ambient game audio keeps pausing your music.",
          value: cfg.audible_threshold ?? 0.01,
          min: 0.001,
          max: 0.2,
          step: 0.001,
          format: (v) => v.toFixed(3),
          onChange: (v) => update({ audible_threshold: v }),
        }),
      ),

      h(
        Group,
        { label: "Voice" },
        h(Slider, {
          key: "vdv",
          label: "Volume while someone is talking",
          hint: "Applies to Discord and other voice apps.",
          value: cfg.voice_duck_volume ?? 0.1,
          min: 0,
          max: 1,
          step: 0.01,
          format: pct,
          onChange: (v) => update({ voice_duck_volume: v }),
        }),
        h(Slider, {
          key: "vrm",
          label: "Recovery time",
          hint: "How long after someone stops talking before volume returns.",
          value: cfg.voice_release_ms ?? 800,
          min: 100,
          max: 4000,
          step: 50,
          format: ms,
          onChange: (v) => update({ voice_release_ms: v }),
        }),
      ),

      h(
        Group,
        { label: "Timing" },
        h(Slider, {
          key: "gam",
          label: "Wait before pausing",
          hint: "Stops a single menu click from pausing your music.",
          value: cfg.game_attack_ms ?? 300,
          min: 0,
          max: 2000,
          step: 50,
          format: ms,
          onChange: (v) => update({ game_attack_ms: v }),
        }),
        h(Slider, {
          key: "grm",
          label: "Wait before resuming",
          hint: "Stops quiet moments in a game from restarting your music too early.",
          value: cfg.game_release_ms ?? 2500,
          min: 200,
          max: 10000,
          step: 100,
          format: ms,
          onChange: (v) => update({ game_release_ms: v }),
        }),
      ),

      h(
        Group,
        { label: "Browsers" },
        h(
          "label",
          { className: "dkf-toggle" },
          h("input", {
            type: "checkbox",
            checked: cfg.browsers_as_games ?? true,
            onChange: (e) => update({ browsers_as_games: e.target.checked }),
          }),
          "Pause for browser audio",
        ),
        h(
          "div",
          { className: "dkf-field-hint" },
          "Covers YouTube and anything else playing in a browser. Windows reports browser audio per process, not per tab, so this cannot tell one tab from another.",
        ),
      ),

      h(
        "div",
        { className: "dkf-footer" },
        h(
          "label",
          { className: "dkf-toggle" },
          h("input", {
            type: "checkbox",
            checked: cfg.enabled ?? true,
            onChange: (e) => update({ enabled: e.target.checked }),
          }),
          "Enabled",
        ),
        h(
          "label",
          { className: "dkf-toggle" },
          h("input", {
            type: "checkbox",
            checked: state.autostart,
            onChange: (e) => {
              state.autostart = e.target.checked;
              send({ type: "autostart", enabled: e.target.checked });
              notify();
            },
          }),
          "Start with Windows",
        ),
        h(
          "div",
          { style: { display: "flex", alignItems: "center", gap: "14px" } },
          h(
            "button",
            {
              className: "dkf-reset",
              onClick: () => {
                send({ type: "reset" });
                announced.clear();
              },
            },
            "Reset decisions",
          ),
          h("span", null, `${state.knownGames} games detected`),
        ),
      ),
    );
  }

  // icon
  const ICON = `
<svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" focusable="false">
  <path fill="currentColor" fill-rule="evenodd" d="M13.9 1.6a4.2 4.2 0 0 0-4.2 4.2c0 .75.2 1.45.54 2.05H6.6c-.6 0-.97-.28-1.24-.78-.3-.58-1.14-.62-1.5-.08A6.5 6.5 0 0 0 2.7 10.7c0 4.3 4.3 7.7 9.5 7.7s9.4-3.4 9.4-7.7c0-2.3-1.2-4.35-3.1-5.75.95-.08 1.85-.45 2.5-1.05.43-.4.35-1.1-.16-1.36-.45-.23-.85-.5-1.15-.83A4.19 4.19 0 0 0 13.9 1.6zM9.5 11.5h2.6c1.5 0 2.72 1.22 2.72 2.72 0 1.5-1.22 2.72-2.72 2.72-1.5 0-3.4-1.35-4.3-2.85-.48-.8.05-1.75 1-1.75z"/>
  <circle cx="15.1" cy="4.75" r="1.05" fill="var(--dkf-icon-hole, #121212)"/>
  <g transform="translate(14.6 14.2) scale(0.92)">
    <circle cx="5.3" cy="5.3" r="5.55" fill="var(--dkf-icon-hole, #121212)"/>
    <path fill="currentColor" d="M9.06 4.35l-1.03-.17a2.9 2.9 0 0 0-.3-.72l.61-.85a.35.35 0 0 0-.04-.45l-.69-.69a.35.35 0 0 0-.45-.04l-.85.61a2.9 2.9 0 0 0-.72-.3L5.42.71A.35.35 0 0 0 5.08.42h-.98a.35.35 0 0 0-.34.29l-.17 1.03a2.9 2.9 0 0 0-.72.3l-.85-.61a.35.35 0 0 0-.45.04l-.69.69a.35.35 0 0 0-.4.45l.61.85a2.9 2.9 0 0 0-.3.72l-1.03.17a.35.35 0 0 0-.29.34v.98c0 .17.12.32.29.34l1.03.17c.7.26.17.5.3.72l-.61.85a.35.35 0 0 0 .4.45l.69.69c.12.12.3.14.45.04l.85-.61c.23.13.47.23.72.3l.17 1.03c.3.17.17.29.34.29h.98a.35.35 0 0 0 .34-.29l.17-1.03c.25-.7.5-.17.72-.3l.85.61c.14.1.33.08.45-.04l.69-.69a.35.35 0 0 0 .04-.45l-.61-.85c.13-.23.23-.47.3-.72l1.03-.17a.35.35 0 0 0 .29-.34v-.98a.35.35 0 0 0-.29-.34zM4.59 6.72a2.13 2.13 0 1 1 0-4.26 2.13 2.13 0 0 1 0 4.26z"/>
  </g>
</svg>`;

  function openPanel() {
    const container = document.createElement("div");
    container.className = "dkf-root";
    Spicetify.ReactDOM.render(h(Settings), container);
    Spicetify.PopupModal.display({
      title: "Duckify",
      content: container,
      isLarge: true,
    });
  }

  // topbar
  function mountButton() {
    if (document.querySelector(".dkf-topbar-btn")) return true;

    const anchor = [...document.querySelectorAll("button")].find(
      (b) => b.getAttribute("aria-label") === "Marketplace",
    );
    if (!anchor?.parentElement) return false;

    const btn = document.createElement("button");
    btn.className = `dkf-topbar-btn ${anchor.className}`;
    btn.setAttribute("aria-label", "Duckify");
    btn.setAttribute("title", "Duckify");
    btn.innerHTML = ICON;
    btn.onclick = openPanel;

    anchor.parentElement.insertBefore(btn, anchor.nextSibling);
    return true;
  }

  // init
  injectStyle();
  connect();

  if (!mountButton()) {
    const observer = new MutationObserver(() => {
      if (mountButton()) observer.disconnect();
    });
    observer.observe(document.body, { childList: true, subtree: true });
  }
})();
