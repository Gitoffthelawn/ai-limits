# Desktop Icon Generation

## Rules

- The master artwork has no internal padding.
- The desktop source adds a black background and desktop-safe padding.
- All desktop platform assets must be derived from the desktop source, never directly from the master artwork.
- Android and iOS icon folders are outside this desktop application scope.

The current packaged icon list is defined in [the Tauri configuration](../../../src-tauri/tauri.conf.json). Asset-generation commands and tool choices belong to the implementation, not this policy.
