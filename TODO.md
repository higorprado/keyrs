# TODO

## Solved
- [x] Hotplug support for device reconnection (KVM switch scenario)
  - Solution: udev monitoring with zero CPU overhead when idle
  - Commits: 56762cc, 09e6a12
- [x] Intermittent terminal regression: after pressing `Cmd+Space` repeatedly, `Ctrl+C` may stop working in terminal.
  - Status: not reliably reproducible yet.
  - Next step: add targeted event/state debug logging around launcher toggles and modifier state cleanup.

## Investigate Later
- [ ] Intermittent newline spam at startup
  - Status: pre-existing intermittent bug where keyrs outputs many newlines at startup, requiring keypresses to stop
  - Likely cause: stale events in device buffer from before grab (e.g., Enter key from running command)
  - Attempted fix: drain events after grab with `fetch_events()` - caused regression in hotplug
  - Next steps: investigate if issue is in Wayland/wl_seat handling, uinput initialization, or add debug logging to trace event sources