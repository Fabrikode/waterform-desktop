/**
 * The two languages, and the two sides.
 *
 * Every reason the Rust half can refuse an address has to have a sentence in
 * both languages, or a customer meets "Could not connect" where we knew exactly
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

describe("shell strings", () => {
  it("finds the failure codes the shell can produce", () => {
    // If this drops to nothing the test below has stopped testing anything.
    expect(codes.length).toBeGreaterThanOrEqual(7);
    expect(codes).toContain("certificate");
    expect(codes).toContain("not-waterform");
  });

  for (const lang of ["tr", "en"]) {
    it(`says something in ${lang} for every one of them`, () => {
      for (const code of codes) {
        expect(dict[lang].errors[code], `${lang}: ${code}`).toBeTruthy();
      }
    });
  }

  it("says the same things in both languages", () => {
    expect(Object.keys(dict.tr).sort()).toEqual(Object.keys(dict.en).sort());
    expect(Object.keys(dict.tr.errors).sort()).toEqual(Object.keys(dict.en.errors).sort());
  });

  it("keeps em dashes out of the copy", () => {
    const all = JSON.stringify(dict);
    expect(all).not.toMatch(/—/);
  });
});
