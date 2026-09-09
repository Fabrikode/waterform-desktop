/**
 * What the shell is, which server it is talking to, and how to reach us.
 *
 * The server and its version are on here because they are the two things we ask
 * for first when something is wrong, and a customer should be able to read them
 * off a window rather than be talked through a settings file.
 */
import { invoke } from "@tauri-apps/api/core";
import { dict } from "./_strings.js";

const el = (id) => document.getElementById(id);

/**
 * The server reports the commit it was built from, which is forty characters of
 * hexadecimal. Eight of them identify it just as well and leave the line
 * readable; anyone who needs the rest is reading it off the server anyway.
 */
function shortVersion(version) {
  if (!version) return null;
  return /^[0-9a-f]{16,}$/.test(version) ? version.slice(0, 8) : version;
}

async function main() {
  const state = await invoke("shell_state");
  const t = dict[state.lang] ?? dict.en;
  document.documentElement.lang = state.lang;
  for (const node of document.querySelectorAll("[data-s]")) {
    node.textContent = t[node.dataset.s];
  }

  el("version").textContent = `${t.aboutVersion} ${state.version}`;
  el("made").textContent = t.aboutMade;
  el("server").textContent = state.serverUrl ?? t.aboutUnset;
  el("server-version").textContent = shortVersion(state.serverVersion) ?? t.aboutUnset;

  const mail = el("mail");
  mail.textContent = t.aboutMailAddress;
  mail.href = `mailto:${t.aboutMailAddress}`;

  // Every address here leaves the application. Opening it in the window would
  // turn the shell into a browser, so the shell hands it to the real one.
  for (const link of document.querySelectorAll(".links a")) {
    link.addEventListener("click", (event) => {
      event.preventDefault();
      invoke("open_link", { url: link.href });
    });
  }

  el("close").addEventListener("click", () => invoke("close_about"));
}

main();
