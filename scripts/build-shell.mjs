/**
 * Builds the shell pages into `dist/`, which is what the binary carries.
 *
 * The shell is three small pages, so there is no framework and no dev server:
 * esbuild is here for one reason only, to turn the `@tauri-apps/api` imports
 * into something a browser can load. Everything else is copied verbatim, which
 * keeps the pages readable in the repository and in the built app.
 */
import { build } from "esbuild";
import { cp, mkdir, readdir, rm } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const src = join(root, "src");
const out = join(root, "dist");

await rm(out, { recursive: true, force: true });
await mkdir(out, { recursive: true });

const entries = (await readdir(join(src, "js")))
  .filter((f) => f.endsWith(".js") && !f.startsWith("_"))
  .map((f) => join(src, "js", f));

await build({
  entryPoints: entries,
  outdir: join(out, "js"),
  bundle: true,
  format: "esm",
  target: "es2022",
  minify: false,
  sourcemap: false,
  logLevel: "warning",
});

for (const name of await readdir(src)) {
  if (name === "js") continue;
  await cp(join(src, name), join(out, name), { recursive: true });
}

console.log(`shell built: ${entries.length} scripts + pages -> dist/`);
