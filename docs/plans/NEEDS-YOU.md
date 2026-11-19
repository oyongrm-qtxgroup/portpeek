# Needs You

- [ ] 2026-08-12 **Decide whether to code-sign the installer.** It is unsigned, so
  Windows SmartScreen shows *"Windows protected your PC"* on first run and users
  must click **More info â†’ Run anyway**. The only real fix is an Authenticode
  certificate (OV ~$200-400/yr, or EV for instant reputation). Reputation also
  builds over time with downloads, but slowly and per-certificate. Documented in
  the README and release notes so nobody is surprised. No action needed if you are
  the only user.

- [ ] 2026-08-12 **Confirm the Docker/WSL badges against real Docker/WSL.** Neither
  is available on this machine: Docker is not installed, and `wsl -l -q` returns no
  distros, so nothing can publish or forward a port.
  What IS verified: detection was exercised end to end against real processes named
  `com.docker.backend.exe` and `wslrelay.exe` holding real listening sockets â€” they
  rendered Docker/WSL badges with the tooltip and no kill button â€” and `vmmem.exe`
  correctly did NOT match. What is NOT verified is the premise that current Docker
  Desktop and WSL2 actually use those process names; that comes from PRD Â§7. PRD Â§12
  already records that the WSL relay name varies by networking mode and Windows build.
  To check: `docker run -p 8085:80 nginx`, or start a dev server inside WSL2, then
  confirm the row shows the badge and no âœ•.

## Resolved

- [x] 2026-08-12 Installer verified end to end: installs per-user to
  `%LOCALAPPDATA%\PortPeek` without admin, creates a Start Menu shortcut, launches,
  and uninstalls cleanly leaving `%APPDATA%\portpeek\config.json` intact so a
  reinstall keeps your pinned ports.

- [x] 2026-08-12 Dev build uninstalled. The stale autostart entry pointing at
  `src-tauri\target\release\portpeek.exe` was removed â€” left in place it would have
  tried to launch a missing exe at every sign-in once the repo build was deleted.

- [x] 2026-08-12 Autostart toggle verified. Header â» button confirmed writing
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\portpeek` =
  `...\portpeek.exe --autostart`, and removing it again. Tray menu event delivery
  separately verified (tray â†’ Quit exits the app).

- [x] 2026-11-19 **Reinstall the MSVC C++ build tools + Windows SDK, then ping me to
  finish the autostart fix.** `link.exe` and `Windows Kits\10\Lib` are gone from this
  machine (both were present in the first session; the `BuildTools` folder and
  `vswhere.exe` are now empty/missing), so Rust cannot link and I could not compile,
  run `cargo test`, or rebuild the installer. The source fix for the unquoted autostart
  path is written, applied and committed, but is **unverified**: the algorithm was
  checked by a line-for-line transcription, not by compiling the Rust.
  Reinstall with:
  `winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"`
  Then I run `cargo test --lib`, rebuild, and verify the registry value end to end.
