/**
 * The three languages, and the two sides.
 *
 * Every reason the Rust half can refuse an address has to have a sentence in
 * every language, or a customer meets "Could not connect" where we knew exactly
 * what was wrong. The list is read out of the Rust source rather than copied,
 * because a copy is a thing that goes stale quietly.
 */
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { dict } from "../src/js/_strings.js";

const rust = readFileSync(resolve(process.cwd(), "src-tauri/src/lib.rs"), "utf8");

function codesFrom(fnName) {
  const body = rust.split(`fn ${fnName}(`)[1]?.split("\n}")[0] ?? "";
  return [...body.matchAll(/=> "([a-z-]+)"/g)].map((m) => m[1]);
}

const codes = [...codesFrom("address_code"), ...codesFrom("probe_code")];

// The languages the Rust half can hand over, read out of `Lang::code` for the
// same reason: a language added there and not here must fail a test.
const i18nRs = readFileSync(resolve(process.cwd(), "src-tauri/src/i18n.rs"), "utf8");
const langs = [
  ...(i18nRs.split("pub fn code(self)")[1]?.split("\n    }")[0] ?? "").matchAll(/=> "([a-z]+)"/g),
].map((m) => m[1]);

describe("shell strings", () => {
  it("finds the failure codes the shell can produce", () => {
    // If this drops to nothing the test below has stopped testing anything.
    expect(codes.length).toBeGreaterThanOrEqual(7);
    expect(codes).toContain("certificate");
    expect(codes).toContain("not-waterform");
  });

  it("knows every language the shell can speak", () => {
    expect(langs).toEqual(["tr", "en", "de"]);
    expect(Object.keys(dict).sort()).toEqual([...langs].sort());
  });

  for (const lang of ["tr", "en", "de"]) {
    it(`says something in ${lang} for every one of them`, () => {
      for (const code of codes) {
        expect(dict[lang].errors[code], `${lang}: ${code}`).toBeTruthy();
      }
    });
  }

  it("says the same things in every language", () => {
    for (const lang of ["en", "de"]) {
      expect(Object.keys(dict[lang]).sort(), lang).toEqual(Object.keys(dict.tr).sort());
      expect(Object.keys(dict[lang].errors).sort(), lang).toEqual(Object.keys(dict.tr.errors).sort());
      for (const [key, value] of Object.entries(dict[lang])) {
        if (typeof value === "string") expect(value.trim(), `${lang}: ${key}`).not.toBe("");
      }
    }
  });

  it("addresses a German reader as Sie, never du", () => {
    const german = JSON.stringify(dict.de);
    expect(german).not.toMatch(/\b(du|dich|dir|dein|deine|deinen)\b/i);
  });

  it("keeps em dashes out of the copy", () => {
    const all = JSON.stringify(dict);
    expect(all).not.toMatch(/—/);
  });
});
