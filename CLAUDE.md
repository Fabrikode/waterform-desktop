# WaterForm Desktop

The window WaterForm runs in on a customer's machine. Read this before changing
anything; it is the contract for how work is done here.

The application itself is not in this repository and never will be. It is at
`Fabrikode/waterform` and it is served from a server. What is here is the window
around it: which server, what happens to a file it hands over, and how this
window replaces itself when there is a newer one. Roughly 900 lines of Rust and
three HTML pages. If a change would add a second place where the product's
behaviour is decided, it belongs on the server instead.

## Why the pieces are what they are

| Choice | Why |
|---|---|
| **Tauri v2**, not Electron | The shell uses no native API beyond a file save and a menu. Tauri ships 5 to 10 MB against Electron's 90+ and borrows the system's web view, so a Chromium security release is the operating system's problem, not a release of ours. |
| **This repository is public** | On a private repository a macOS runner minute counts ten times and a Windows one twice; one release would take a tenth of the organisation's 2,000 monthly minutes and share that budget with the application's own CI. Public runners and public releases are free. Nothing here is worth hiding. There is no open source licence: the code is readable, not reusable. |
| **No Apple Developer account** | Ad-hoc signing, no notarisation. A customer gets one Gatekeeper prompt on first install, which the download page walks them through, and never again, including after updates. Adding the paid account later changes CI secrets and nothing else. |
| **No offline mode** | Without a server there is no application. A local copy of a customer's tanks would be a second version of the truth about something they build out of steel. |
| **Settings in a JSON file** | Four fields. A support engineer can read it down the phone; a corrupt one costs a re-typed address, not a reinstall. |

## The rules

- **The page the server serves gets nothing.** `capabilities/shell.json` has no
  `remote` block, so every Tauri command is refused on any page that did not come
  out of this binary. Proven, not assumed: with the bridge object present on the
  page, `shell_state`, `connect`, `restart_now` and `plugin:opener|open_url` were
  all refused. Adding a `remote` block would hand a flaw in a page on the server
  the keys to the customer's machine. Do not add one.
- **Nothing is injected into the application's page.** No banner, no script, no
  `eval`. The update offer is a window of ours; a reload is a navigation, not an
  injected `location.reload()`. The moment the shell writes script into a remote
  document, the boundary above is decoration.
- **Windows are shell pages or the application, never both.** `connect.html`,
  `offline` states and `update.html` are ours and get IPC. The server's pages get
  a window and nothing else.
- **Every state says which server and why.** "Could not connect" without an
  address is a support call. The reasons live as codes in Rust
  (`address_code`, `probe_code`) and as sentences in `src/js/_strings.js`; a test
  reads the codes out of the Rust source and fails if either language is missing
  one.
- **Turkish and English, both, everywhere a person reads.** The shell follows the
  machine's language until the customer chooses otherwise in the menu, and then
  it follows the choice; the application follows the account's language, which
  is a different setting on a different side. The choice exists because the
  machine is a poor guess here: a Turkish engineer is routinely handed an
  English Windows install by whoever set the office up. Changing it redraws the
  menu and reloads any shell page that is open, so the setting takes effect in
  front of the person who changed it. No em dashes.
- **One version in three files.** `tauri.conf.json` is the source of truth;
  `npm run version:set -- x.y.z` moves all three and `npm run version:check`
  fails when they drift. The release workflow refuses a version that already has
  a tag, because a release nobody is offered is worse than no release.
- **The signing key is not in here.** `TAURI_SIGNING_PRIVATE_KEY` is a repository
  secret and a password-manager entry. **If it is lost, every installed copy
  stops updating for good** and has to be reinstalled by hand. The public half
  is in `tauri.conf.json` and is meant to be there.
- **Tests ship with the code**, as everywhere in Fabrikode. Rust for anything
  with a decision in it (address normalising, the health answer, settings,
  snoozing); Vitest for the shell pages. Browser-level behaviour that only a real
  web view can answer is in `RELEASE-CHECKLIST.md` instead of being faked.

## Layout

```
src/                     the shell's own pages: plain HTML, one stylesheet
  connect.html           first run, the server setting, and "it did not answer"
  update.html            the update offer, its progress, and the restart
  about.html             version, which server, and how to reach us
  js/                    esbuild's only job is the @tauri-apps/api imports
src-tauri/src/
  lib.rs                 windows, downloads, navigation, the commands
  server.rs              what counts as an address, and is it WaterForm  [tested]
  settings.rs            the four things we remember                     [tested]
  update.rs              check, offer, install, snooze
  menu.rs                the native menu
  i18n.rs                TR/EN for what the operating system draws       [tested]
tests/                   the shell pages under jsdom
.github/workflows/       check (dev, Linux only) and release (release, four targets)
```

## The About window

Our own, not the platform's About panel, because the panel cannot carry an
address anyone can click and the two things a customer is asked for when
something is wrong are which server they are on and what version is running
there. Both are on it, beside the product page, fabrikode.com and the contact
address for the language in force. Every link leaves through the system browser:
the window has a navigation guard, so it cannot wander onto a website and stop
being chrome.

## What the shell knows about the application

Only `/api/health`, which already returns `app`, `ok`, `version` and `mode`. That
is the whole contract. If the application ever needs to know it is running inside
the shell, the page can read `window.isTauri`, which Tauri sets even where it
grants nothing else. The user agent is deliberately **not** customised: Tauri
replaces it rather than appending, and inventing a browser string would be
shipping a lie that ages.

## Things that are known and deliberate

- `window.open('', '_blank')` returns null on macOS. The engine has one such
  fallback for a PDF preview and it is dead code there, because `window.wfPdf`
  exists in the web application and wins. If a page ever needs a real second
  window, it has to be built as one, not allowed.
- Downloads land in the downloads folder without a save dialog. The dialog would
  have to be answered on the web view's own thread, and Chrome's behaviour is
  what people expect. A notification says where the file went; the File menu
  opens the folder.
- The shell only checks whether the server is reachable **at startup**. If the
  network drops while the application is open, the page stays as it is: replacing
  a working page with an "offline" screen would throw away work the customer has
  not saved. Reload is in the menu.
