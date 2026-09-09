# WaterForm Desktop

WaterForm in its own window. The application itself lives on a server; this is
the window around it, and it is the whole of this repository.

Downloads for customers: **https://waterform.fabrikode.com/downloads**

## What it does

- Asks which WaterForm server to use, checks the answer, and remembers it. The
  default is `https://app.waterform.fabrikode.com`; a company running WaterForm
  on its own server puts that address in instead, from the menu.
- Puts Excel, PDF and DXF exports in the downloads folder and says so.
- Updates itself. A new release on the `release` branch reaches installed copies
  within a few hours, or at the next launch.

There is no offline mode. Without a server there is no application, and
pretending otherwise would mean two versions of the truth about a customer's
tanks.

## Working on it

```bash
npm install
npm run dev        # builds the shell pages, then runs the app
npm run verify     # what CI runs: versions, shell tests, fmt, clippy, cargo test
```

Rust is needed once: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`.
On Linux also `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`.

## Releasing

```bash
npm run version:set -- 1.1.0     # one version, three files
git commit -am "Raise the version to 1.1.0"
git push origin dev              # check must be green
git checkout release && git merge dev && git push origin release
```

That is the release. CI builds four targets, publishes a GitHub release with the
installers and a signed `latest.json`, and installed copies pick it up. See
`RELEASE-CHECKLIST.md` for what to try before merging, and `CLAUDE.md` for why
things are the way they are.
