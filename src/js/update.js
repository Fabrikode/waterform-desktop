/**
 * The update window.
 *
 * Its own window on purpose. The alternative, a banner inside the page the
 * server serves, would mean the shell writing script into a remote document,
 * and an update that interrupts a drawing half way through.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { dict } from "./_strings.js";

const el = (id) => document.getElementById(id);
let t = dict.en;

function buttons(primary, secondary) {
  for (const [node, spec] of [
    [el("ok"), primary],
    [el("no"), secondary],
  ]) {
    node.hidden = !spec;
    node.onclick = spec ? spec.run : null;
    if (spec) node.textContent = spec.text;
  }
  // This window opens on top of whatever the customer was doing, so the thing it
  // is asking about should be one key away rather than one hunt away.
  if (!el("ok").hidden) {
    el("ok").focus();
  }
}

function render(status) {
  if (!status) return;
  el("bar").hidden = true;

  switch (status.state) {
    case "checking":
      el("title").textContent = t.checking;
      el("body").textContent = "";
      buttons(null, null);
      break;

    case "upToDate":
      el("title").textContent = t.upToDateTitle;
      el("body").textContent = t.upToDateBody;
      buttons({ text: t.close, run: () => invoke("close_update") }, null);
      break;

    case "available":
      el("title").textContent = t.availableTitle(status.version);
      el("body").textContent = t.availableBody;
      buttons(
        { text: t.update, run: () => invoke("start_update") },
        { text: t.later, run: () => invoke("later_update") },
      );
      break;

    case "downloading": {
      el("title").textContent = t.downloading;
      const pct = status.total ? Math.round((status.received / status.total) * 100) : null;
      el("body").textContent = pct === null ? "" : `${pct}%`;
      el("bar").hidden = false;
      el("fill").style.width = pct === null ? "100%" : `${pct}%`;
      buttons(null, null);
      break;
    }

    case "ready":
      el("title").textContent = t.readyTitle;
      el("body").textContent = t.readyBody;
      buttons({ text: t.restart, run: () => invoke("restart_now") }, null);
      break;

    case "failed":
      el("title").textContent = t.failedTitle;
      // The reason as the updater gave it. A customer forwards this sentence to
      // us, so a translated summary would lose the only useful part of it.
      el("body").textContent = status.message;
      buttons({ text: t.close, run: () => invoke("close_update") }, null);
      break;
  }
}

async function main() {
  const state = await invoke("shell_state");
  t = dict[state.lang] ?? dict.en;
  document.documentElement.lang = state.lang;
  await listen("wf://update", (event) => render(event.payload));
  render(await invoke("update_status"));
}

main();
