# Needs You

_Nothing outstanding._

## Resolved

- [x] 2026-08-12 Autostart toggle verified. Header ⏻ button confirmed writing
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\portpeek` =
  `...\portpeek.exe --autostart`, and removing it again. Tray menu event delivery
  separately verified (tray → Quit exits the app), and the tray item calls the same
  `set_autostart`, so no untested code remains between them.
  Note: autostart registers whichever exe path is running. If you install via
  MSI/NSIS and delete the repo build, re-toggle from the installed copy.
