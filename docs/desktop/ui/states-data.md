# Tauri UI States Data Contract

Backend state:

- selected by `noFreshData: true`.
- shown when `limits` is empty.
- does not use `errorMessage` text in this state.

Frontend-only state:

- inline source priority control state comes from `appSettings.sourcePriority`.
- help view open/closed state and the selected help chapter are frontend state.
