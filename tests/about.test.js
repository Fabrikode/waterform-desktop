/**
 * The About window.
 *
 * It is two facts and three addresses, and each of them is here because a
 * customer is asked for it when something goes wrong. The tests hold the two
 * things that would quietly stop being true: that an unconfigured machine says
 * so rather than showing an empty line, and that no address on this window ever
 * opens inside the shell.
 */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args) => invoke(...args) }));

const html = readFileSync(resolve(process.cwd(), "src/about.html"), "utf8");
const body = html.split("<body>")[1].split("<script")[0];

async function open(state) {
  document.body.innerHTML = body;
  invoke.mockReset();
  invoke.mockImplementation((command) => {
    if (command === "shell_state") return Promise.resolve(state);
    return Promise.resolve();
  });
  vi.resetModules();
  await import("../src/js/about.js");
  await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("shell_state"));
  await new Promise((r) => setTimeout(r, 0));
}

const base = {
  lang: "tr",
  platform: "macos",
  version: "1.1.0",
  defaultServer: "https://app.waterform.fabrikode.com",
  serverUrl: "https://depo.sirket.com",
  serverVersion: "0c74ba8",
};

describe("the about window", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  it("names the shell's version and the server behind it", async () => {
    await open(base);
    expect(document.getElementById("version").textContent).toBe("Sürüm 1.1.0");
    expect(document.getElementById("server").textContent).toBe("https://depo.sirket.com");
    expect(document.getElementById("server-version").textContent).toBe("0c74ba8");
  });

  it("says so when there is no server yet instead of showing a blank", async () => {
    await open({ ...base, serverUrl: null, serverVersion: null });
    expect(document.getElementById("server").textContent).toBe("ayarlanmadı");
    expect(document.getElementById("server-version").textContent).toBe("ayarlanmadı");
  });

  it("writes to the Turkish address in Turkish and the English one in English", async () => {
    await open(base);
    expect(document.getElementById("mail").href).toBe("mailto:iletisim@fabrikode.com");
    await open({ ...base, lang: "en" });
    expect(document.getElementById("mail").href).toBe("mailto:contact@fabrikode.com");
    expect(document.getElementById("version").textContent).toBe("Version 1.1.0");
  });

  it("hands every address to the browser and never follows one itself", async () => {
    await open(base);
    const links = [...document.querySelectorAll(".links a")];
    expect(links).toHaveLength(3);
    for (const link of links) {
      const event = new MouseEvent("click", { cancelable: true, bubbles: true });
      link.dispatchEvent(event);
      expect(event.defaultPrevented).toBe(true);
      expect(invoke).toHaveBeenCalledWith("open_link", { url: link.href });
    }
  });
});

describe("the server version on the about window", () => {
  it("shortens a commit hash and leaves anything else alone", async () => {
    await open({ ...base, serverVersion: "0c74ba8d67401dc61267b6c177d31f46f0329e84" });
    expect(document.getElementById("server-version").textContent).toBe("0c74ba8d");
    await open({ ...base, serverVersion: "2.1.4" });
    expect(document.getElementById("server-version").textContent).toBe("2.1.4");
  });
});
