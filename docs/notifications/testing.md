# Notification Testing

## Testing

General testing entry point: [Testing](../testing/testing.md).

Manual testing can use fake notification triggers without provider data:

```text
ai-limits --test-notification=75
ai-limits --test-notification=50
ai-limits --test-notification=25
ai-limits --test-notification=10
```

These commands should request delivery through the Tauri notifications adapter.

When the Tauri application is unavailable, test notification commands should complete without sending a system notification and without printing an extra notification message.

Trigger calculation is covered with unit tests using fake structured data.

---

## Platform Scope

Development targets:

- macOS
- Windows
- Linux

The target delivery adapter for every supported desktop platform is Tauri notifications.

Initial development is checked directly on macOS. Windows and Linux behavior must be tested later by external testers who have access to those systems.
