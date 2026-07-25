# Notifications

This document defines the target behavior for system notifications.

Notification content, branding, and UI presentation are documented in [notification-content.md](notification-content.md). Testing and platform scope are documented in [notification-testing.md](notification-testing.md).

---

## Goal

The application notifies the user when an important limit event requires attention.

Notifications are a shared product capability. They are used by the desktop interface and can be requested from the terminal interface.

---

## Architecture

The application uses one common notification domain model.

Provider and limit logic produce notification candidates from structured source data. The notification rules stay in the shared Rust core and must not be duplicated in the Tauri frontend.

Notification delivery is separate from notification logic.

The target delivery adapter is Tauri notifications. Platform-specific notification behavior for macOS, Windows, and Linux is delegated to Tauri unless a supported platform later requires product behavior that Tauri cannot provide.

```text
shared core
  structured source data
  notification thresholds
  notification text
  dedupe keys

delivery adapter
  Tauri notifications
  system notification permission
  native system notification delivery
  application icon
  notification click behavior

terminal interface
  existing terminal UI
  optional request to the installed/running Tauri application
```

The shared core must not depend on Tauri or any operating-system notification API. It produces notification candidates and passes them to the delivery layer.

The application does not maintain separate first-party macOS, Windows, and Linux notification adapters in the current target architecture. If Tauri notifications later cannot provide required product behavior on a supported platform, platform-specific delivery adapters may be introduced behind the same delivery interface.

The terminal interface does not send native operating-system notifications directly. When a terminal run needs a system notification, it requests delivery from the installed and available Tauri application.

If the terminal interface cannot hand the notification request to Tauri, it silently skips the system notification. It must not print an additional terminal message, because the terminal UI already contains the relevant information and extra text would not attract attention.

There is no separate macOS helper/notifier in the target architecture.

---

## Delivery Rules

System notifications are delivered through the Tauri notifications adapter.

Rules:

- when the Tauri application is active or minimized, eligible notifications should be delivered as native system notifications through Tauri
- when the terminal interface can reach the installed and available Tauri application, eligible notifications can be delivered through Tauri
- when Tauri is unavailable, the notification is skipped without additional terminal output
- the application does not use a separate notification helper process

The user-facing notification setting controls whether notification checks are enabled.

---

## Click Behavior

Clicking a system notification opens or focuses the Tauri application.

Notification clicks do not need to navigate to a specific provider, limit, event, or screen in the current target behavior.

---

## Deduplication

Rules:

- notifications do not replace each other; every delivered notification is kept as a separate system notification
- the same running process should not repeatedly send the same notification
- notifications are independent for each provider, for example Codex, Claude, and Cursor
- notifications are independent for each called data source
- if different data sources return different limit data for the same provider, this is acceptable
- each called and enabled source is evaluated separately and can produce its own notification candidate

---

## Calculation

Notification triggers are calculated from structured data.

Structured data is used because it is standardized and easier to process consistently across providers and sources.

Notification calculation is independent from the delivery channel. The same candidate generation rules apply whether the request originates from the Tauri UI or from the terminal interface.
