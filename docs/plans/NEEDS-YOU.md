# Needs You

- [ ] 2026-08-12 Confirm the Docker/WSL badges against a **real** Docker container and
  WSL2 dev server when you have them. Neither is available on this machine: Docker
  is not installed, and `wsl -l -q` returns no distros, so nothing can publish or
  forward a port.
  What IS verified: detection was exercised end to end against real processes named
  `com.docker.backend.exe` and `wslrelay.exe` holding real listening sockets — they
  rendered Docker/WSL badges with the tooltip and no kill button — and `vmmem.exe`
  correctly did NOT match. What is NOT verified is the premise that current Docker
  Desktop and WSL2 actually use those process names; that comes from PRD §7, not
  from anything observed here. PRD §12 already records that the WSL relay name
  varies by networking mode and Windows build.
  To check: publish a container port (`docker run -p 8085:80 nginx`) or start a dev
  server inside WSL2, then confirm the row shows the badge and no ✕.

## Resolved

- [x] 2026-08-12 Autostart toggle verified. Header ⏻ button confirmed writing
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\portpeek` =
  `...\portpeek.exe --autostart`, and removing it again. Tray menu event delivery
  separately verified (tray → Quit exits the app).
  Note: autostart registers whichever exe path is running. If you install via
  MSI/NSIS and delete the repo build, re-toggle from the installed copy.
