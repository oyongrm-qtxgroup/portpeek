# Needs You

- [ ] 2026-08-12 Confirm the **tray** "Start with Windows" item writes the registry.
  The header ⏻ button is verified working (writes and removes
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\portpeek` correctly), and tray
  menu events are verified firing (tray → Quit exits the app). The tray item routes
  through the same verified `set_autostart` function, so it should behave identically
  — but that exact click was never proven end to end, because automating the Windows 11
  hidden-icon flyout was too flaky to trust.
  To confirm: right-click the tray icon → "Start with Windows", then run
  `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v portpeek`.
  Prefer the header ⏻ button — it is the verified path and far more discoverable.
