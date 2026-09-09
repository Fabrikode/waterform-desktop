/**
 * The server screen, in the states that do not navigate.
 *
 * The two that do (a reachable server, and a successful connect) end in
 * `location.href`, which jsdom has no answer for; they are covered by the
 * release checklist on a real machine instead.
 */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args) => invoke(...args) }));

const html = readFileSync(resolve(process.cwd(), "src/connect.html"), "utf8");
const body = html.split("<body>")[1].split("<script")[0];

const state = {
  lang: "tr",
  platform: "macos",
  version: "0.1.0",
  defaultServer: "https://app.waterform.fabrikode.com",
  serverUrl: null,
};

async function open(bootstrap, { search = "", serverUrl = null } = {}) {
  document.body.innerHTML = body;
  window.history.replaceState({}, "", `/connect.html${search}`);
  invoke.mockReset();
  invoke.mockImplementation((command) => {
    if (command === "shell_state") return Promise.resolve({ ...state, serverUrl });
    if (command === "bootstrap") return Promise.resolve(bootstrap);
    return Promise.resolve();
  });
  vi.resetModules();
  await import("../src/js/connect.js");
  await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("shell_state"));
  await new Promise((r) => setTimeout(r, 0));
}

const shown = () =>
  ["busy", "form", "down"].filter((id) => !document.getElementById(id).hidden);

describe("the server screen", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("asks for an address on a machine that has never had one", async () => {
    await open({ kind: "needsServer" });
    expect(shown()).toEqual(["form"]);
    expect(document.getElementById("url").value).toBe(state.defaultServer);
    // Nothing to go back to, so there is no way back.
    expect(document.getElementById("cancel").hidden).toBe(true);
  });

  it("names the server and the reason when it does not answer", async () => {
    await open({ kind: "unreachable", url: "https://depo.sirket.com", code: "certificate" });
    expect(shown()).toEqual(["down"]);
    expect(document.getElementById("down-addr").textContent).toBe("https://depo.sirket.com");
    expect(document.getElementById("down-why").textContent).toMatch(/sertifika/i);
  });

  it("falls back to a plain sentence for a code it does not know", async () => {
    await open({ kind: "unreachable", url: "https://depo.sirket.com", code: "something-new" });
    expect(document.getElementById("down-why").textContent).toBe("Bağlanılamadı.");
  });

  it("does not check the server when the menu opened it", async () => {
    // The person came to change the address. Checking the old one and being sent
    // back to it is the one thing they did not ask for.
    await open({ kind: "ready" }, { search: "?change", serverUrl: "https://depo.sirket.com" });
    expect(invoke).not.toHaveBeenCalledWith("bootstrap");
    expect(shown()).toEqual(["form"]);
    expect(document.getElementById("url").value).toBe("https://depo.sirket.com");
    expect(document.getElementById("cancel").hidden).toBe(false);
  });

  it("keeps the form usable after the server refuses the address", async () => {
    await open({ kind: "needsServer" });
    invoke.mockImplementation((command) => {
      if (command === "connect") return Promise.reject("insecure");
      return Promise.resolve();
    });
    document.getElementById("f").dispatchEvent(new Event("submit"));
    await vi.waitFor(() => expect(document.getElementById("err").hidden).toBe(false));
    expect(document.getElementById("err").textContent).toMatch(/https/);
    expect(document.getElementById("go").disabled).toBe(false);
  });
});
