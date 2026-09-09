/**
 * Where this installation is pointed, and what to say when it points nowhere.
 *
 * The window always opens here, even when a server is already configured: the
 * check happens first and the page hands over to the application only once the
 * server has answered. That costs a fraction of a second on a good connection
 * and it is what stops a customer on a broken one from meeting the web view's
 * own error page, which names no address and offers nothing to do about it.
 */
import { invoke } from "@tauri-apps/api/core";
import { dict } from "./_strings.js";

const el = (id) => document.getElementById(id);
const sections = ["busy", "form", "down"];
const show = (name) => sections.forEach((s) => (el(s).hidden = s !== name));

let t = dict.en;
let state;

function label(code) {
  return t.errors[code] ?? t.errors.unknown;
}

function showForm({ value, cancellable }) {
  el("url").value = value;
  el("cancel").hidden = !cancellable;
  el("err").hidden = true;
  el("go").disabled = false;
  el("go").textContent = t.connect;
  show("form");
  el("url").focus();
  el("url").select();
}

function showDown(url, code) {
  el("down-why").textContent = label(code);
  el("down-addr").textContent = url;
  show("down");
}

async function decide() {
  el("busy-addr").textContent = state.serverUrl ?? "";
  show("busy");
  const result = await invoke("bootstrap");
  if (result.kind === "ready") {
    location.href = result.url;
  } else if (result.kind === "unreachable") {
    showDown(result.url, result.code);
  } else {
    showForm({ value: state.defaultServer, cancellable: false });
  }
}

async function submit(event) {
  event.preventDefault();
  const go = el("go");
  el("err").hidden = true;
  go.disabled = true;
  go.textContent = t.connectingButton;
  try {
    const { url } = await invoke("connect", { url: el("url").value });
    location.href = url;
  } catch (code) {
    el("err").textContent = label(String(code));
    el("err").hidden = false;
    go.disabled = false;
    go.textContent = t.connect;
  }
}

async function main() {
  state = await invoke("shell_state");
  t = dict[state.lang] ?? dict.en;
  document.documentElement.lang = state.lang;
  for (const node of document.querySelectorAll("[data-s]")) {
    node.textContent = t[node.dataset.s];
  }

  el("f").addEventListener("submit", submit);
  el("cancel").addEventListener("click", () => invoke("cancel_connect"));
  el("retry").addEventListener("click", decide);
  el("change").addEventListener("click", () =>
    showForm({ value: state.serverUrl ?? state.defaultServer, cancellable: true }),
  );

  // Opened from the menu rather than at startup: the customer came here to
  // change the address, so asking the current server whether it is well and
  // then leaving would be the one thing they did not ask for.
  if (new URLSearchParams(location.search).has("change")) {
    showForm({ value: state.serverUrl ?? state.defaultServer, cancellable: state.serverUrl !== null });
    return;
  }
  await decide();
}

main();
