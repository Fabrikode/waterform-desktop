/**
 * One version, three files.
 *
 * `tauri.conf.json` is the source of truth: it is what the built app reports and
 * what the updater compares against. `package.json` and `Cargo.toml` have to
 * agree, so this either sets all three (`npm run version:set -- 1.1.0`) or, with
 * `--check`, fails when they have drifted. CI runs the check, because a release
 * built from mismatched numbers is a release nobody can reason about.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const confPath = join(root, "src-tauri", "tauri.conf.json");
const pkgPath = join(root, "package.json");
const cargoPath = join(root, "src-tauri", "Cargo.toml");

const conf = JSON.parse(readFileSync(confPath, "utf8"));
const pkg = JSON.parse(readFileSync(pkgPath, "utf8"));
const cargo = readFileSync(cargoPath, "utf8");
const cargoVersion = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const arg = process.argv[2];

if (arg === "--check" || arg === undefined) {
  const all = { "tauri.conf.json": conf.version, "package.json": pkg.version, "Cargo.toml": cargoVersion };
  const distinct = [...new Set(Object.values(all))];
  if (distinct.length !== 1) {
    console.error("Version mismatch:");
    for (const [file, v] of Object.entries(all)) console.error(`  ${file}: ${v}`);
    console.error("Run: npm run version:set -- <version>");
    process.exit(1);
  }
  console.log(`version ${distinct[0]} (three files agree)`);
  process.exit(0);
}

if (!/^\d+\.\d+\.\d+$/.test(arg)) {
  console.error(`Not a version: ${arg}. Expected x.y.z`);
  process.exit(1);
}

conf.version = arg;
pkg.version = arg;
writeFileSync(confPath, `${JSON.stringify(conf, null, 2)}\n`);
writeFileSync(pkgPath, `${JSON.stringify(pkg, null, 2)}\n`);
writeFileSync(cargoPath, cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${arg}"`));
console.log(`version set to ${arg} in three files`);
