# Needs You

- [ ] 2026-08-12 Verify the tray "Start with Windows" toggle actually registers autostart.
  Right-click the PortPeek tray icon (it sits under "Show Hidden Icons") → click
  "Start with Windows". Then confirm a `portpeek` value appears under
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` pointing at the installed exe
  with a `--autostart` argument. Click again to confirm it is removed.
  Everything either side of this is verified: the plugin is wired, autostart defaults
  to OFF (registry checked clean), and launching with `--autostart` correctly parks the
  app in the tray with no visible window. Only the menu-click → registry-write step is
  unproven — automating a Win32 tray popup menu proved unreliable and was abandoned.
