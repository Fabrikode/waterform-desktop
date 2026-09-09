/**
 * The update window's six states.
 *
 * Each one has to say what happened and offer exactly the buttons that make
 * sense there: an offer that cannot be postponed, or a failure with no way to
 * close it, is how a small window becomes something the customer force-quits.
 */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
let emit;

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args) => invoke(...args) }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (_name, handler) => {
    emit = (payload) => handler({ payload });
    return Promise.resolve(() => {});
  },
}));

const html = readFileSync(resolve(process.cwd(), "src/update.html"), "utf8");
const body = html.split("<body>")[1].split("<script")[0];

async function open(initial) {
  document.body.innerHTML = body;
  invoke.mockReset();
  invoke.mockImplementation((command) => {
    if (command === "shell_state") return Promise.resolve({ lang: "tr" });
    if (command === "update_status") return Promise.resolve(initial ?? null);
    return Promise.resolve();
  });
  vi.resetModules();
  await import("../src/js/update.js");
  await vi.waitFor(() => expect(emit).toBeTypeOf("function"));
  await new Promise((r) => setTimeout(r, 0));
}

const title = () => document.getElementById("title").textContent;
const ok = () => document.getElementById("ok");
const no = () => document.getElementById("no");

describe("the update window", () => {
  beforeEach(() => {
    emit = undefined;
    document.body.innerHTML = "";
  });

  it("offers the new version with a way to put it off", async () => {
    await open();
    emit({ state: "available", version: "1.2.0", current: "1.1.0", notes: null });
    expect(title()).toContain("1.2.0");
    expect(ok().hidden).toBe(false);
    expect(no().hidden).toBe(false);

    no().click();
    expect(invoke).toHaveBeenCalledWith("later_update");
    ok().click();
    expect(invoke).toHaveBeenCalledWith("start_update");
  });

  it("shows how far the download has got", async () => {
    await open();
    emit({ state: "downloading", received: 25, total: 100 });
    expect(document.getElementById("bar").hidden).toBe(false);
    expect(document.getElementById("fill").style.width).toBe("25%");
    // Nothing to press while it is working.
    expect(ok().hidden).toBe(true);
    expect(no().hidden).toBe(true);
  });

  it("does not divide by a size the server did not send", async () => {
    await open();
    emit({ state: "downloading", received: 25, total: null });
    expect(document.getElementById("body").textContent).toBe("");
    expect(document.getElementById("fill").style.width).toBe("100%");
  });

  it("asks for a restart once it is installed", async () => {
    await open();
    emit({ state: "ready" });
    ok().click();
    expect(invoke).toHaveBeenCalledWith("restart_now");
  });

  it("repeats the updater's own words when it fails, and can be closed", async () => {
    await open();
    emit({ state: "failed", message: "Read-only file system (os error 30)" });
    expect(document.getElementById("body").textContent).toContain("os error 30");
    ok().click();
    expect(invoke).toHaveBeenCalledWith("close_update");
  });

  it("answers a check that found nothing", async () => {
    await open({ state: "upToDate" });
    expect(title()).toBe("Güncel");
    expect(no().hidden).toBe(true);
  });
});
