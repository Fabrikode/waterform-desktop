# Before merging `dev` into `release`

`npm run verify` and the `check` workflow cover what a machine can judge. These
are the things only a person on a real machine can: the web views differ, and
the ones that differ are exactly the ones the exports go through.

Build locally with `npm run build` (or take the artefacts from the last release
build) and go through the list on whichever platforms the change could touch. A
change to the Rust half is all three; a change to a shell page is one.

## Every platform

- [ ] First run with no `settings.json`: the server screen appears with the
      default address, connects, and the sign-in page loads.
- [ ] A wrong address says why: `http://waterform.fabrikode.com` (refused before
      it is even tried), `https://example.com` (not WaterForm), an address that
      does not resolve.
- [ ] Menu > Server: the address can be changed and cancelled. Cancel returns to
      the application rather than closing it.
- [ ] Sign in, open a job, **export the Excel analysis**: the file lands in the
      downloads folder, opens in Excel, and a notification says where it went.
- [ ] **Export the A0 sheet as PDF** and **the DXF**. Same three checks.
- [ ] Export the same file twice: the second is ` (2)`, nothing is overwritten.
- [ ] The 3D view turns, and the connections can be placed by clicking.
- [ ] Copy and paste work in a text field (macOS: this is the Edit menu; without
      it ⌘C does nothing).
- [ ] A link to somewhere that is not the server opens in the browser, not in
      the window.
- [ ] Menu > Check for updates on the newest version says so and closes.
- [ ] Menu > Language: switching to Turkish redraws the menu at once, and an
      About window that is already open changes with it. The choice survives a
      restart; "System language" puts it back.
- [ ] Menu > Language > Deutsch: every menu the platform shows is German (on a
      Mac: Ablage, Bearbeiten, Darstellung, Einsetzen, Im Dock ablegen; on
      Windows and Linux: Datei, Bearbeiten, Ansicht, Einfügen, Minimieren),
      nothing is cut off, and the About, server and update windows are German
      too ("Sie", no leftover English or Turkish). A machine whose system
      language is German opens in German with "System language" ticked.
- [ ] **Print the engineering calculation** with the page's own print button:
      the system's print dialog opens over the window, the preview shows the
      whole calculation on **one A4 page** with a margin and nothing clipped,
      and it prints (or saves as PDF) the same. After the dialog closes, by
      printing and again by cancelling, the page on screen is back to normal
      at once: sections that were closed are closed again and nothing is still
      scaled down.
- [ ] File > Print (⌘P / Ctrl+P) on the engineering page does the same as the
      button, **once** (Windows: one dialog, not the web view's own print
      preview on top of ours or instead of it). On another page of the
      application it prints that page. On the server screen it does nothing.
- [ ] Printing never opens a browser, a "which application should open
      wf-desktop" question, or an error page.
- [ ] About: the version, the server address and the server's version are all
      right, and the three links open in the browser rather than in the window.
      The contact address matches the language.

## macOS only

- [ ] The downloaded DMG, opened for the first time, is refused by Gatekeeper.
      **This is expected.** Follow the download page's own instructions
      (System Settings > Privacy & Security > Open Anyway) and confirm they are
      still accurate for the current macOS.
- [ ] After an update installs, the application reopens **without** asking again.
      Verified 2026-09-09 on macOS 26.6: the replaced bundle carries no
      quarantine flag, so it should stay true; it is on this list because it is
      the one thing that would make updates unusable if Apple changed it.
- [ ] The application menu carries About, Server, Check for updates, Hide, Quit.
- [ ] Printing: the print sheet slides down from the window, the rest of the
      Mac stays usable while it is up, and a second ⌘P while it is up does not
      open a second one. The preview is not blank.

## Windows only

- [ ] SmartScreen warns on the installer. Expected; the download page says so.
- [ ] The installer does not ask for administrator rights (it installs per user).
- [ ] An update installs without a UAC prompt.
- [ ] The installer's language follows Windows, including German.

## Linux only

- [ ] The AppImage runs after `chmod +x` on Ubuntu 22.04 or newer.
- [ ] The 3D view renders. If it does not, the shell already sets
      `WEBKIT_DISABLE_DMABUF_RENDERER`; note which distribution and driver.
- [ ] The `.deb` installs and appears in the applications menu.

## Then

- [ ] `npm run version:set -- x.y.z`, commit, push `dev`, wait for green.
- [ ] Merge into `release`. Watch the four builds.
- [ ] Open the published release: five installers, four `.tar.gz`/`.sig` pairs
      and `latest.json`.
- [ ] An installed older copy offers the update within six hours, or at once if
      you restart it.
- [ ] The downloads page shows the new version within ten minutes.
