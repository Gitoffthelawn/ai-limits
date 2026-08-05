# macOS Menu Bar Popover

## Status

Native macOS tray/window logic now exists — the Popover is reachable in a real build: the menu bar tray icon toggles a real `"popover"` `WebviewWindow` showing the real [popover.html](../../frontend/popover.html) surface. The `[info]`/`[gear]`/"Open Application" bridge globals are real (real Tauri commands, not stubs — see [Toolbar](#toolbar) and [Entry Points](#entry-points)); display/provider-visibility settings and manual theme changes sync live from the Main Window into an already-open Popover (see [Cross-Window Sync](#cross-window-sync)). The window carries a native `NSVisualEffectView` vibrancy material and stays on whatever Space is currently active instead of switching Spaces — see [Native Look and Space Behavior](#native-look-and-space-behavior). The webview's own visual layer was rebuilt to read as a system panel rather than a shrunken Main Window — see [Visual Layer](#visual-layer). The native frame behaves like a menu bar panel too: it floats above other windows, carries a real `NSWindow` shadow, anchors under the clicked tray icon on the right display without running off screen, and sizes its height to the content the frontend reports — see [Window Frame](#window-frame), [Positioning](#positioning) and [Window Size](#window-size). The menu bar icon is a purpose-built monochrome template image with a right-click menu — see [Menu Bar Icon](#menu-bar-icon). Remaining gaps are tracked at the bottom under [Open Items](#open-items) — notably the Popover still has no native arrow/tip pointing at the tray icon.

## Intent

Add a second surface for macOS: a **Menu Bar Popover**, alongside the existing **Main Window**. Both exist at the same time and serve different jobs — the Popover is not a replacement for the Main Window, and there is no setting that switches between them. The Popover is the same application, the same backend logic, and the same three provider cards (Cursor, Claude, Codex), rendered through a smaller, simpler UI.

- Same data, same refresh logic, same settings model as the Main Window.
- Simpler layout than the Main Window: no Help entry point in the Popover.
- The three provider cards keep their content model from [provider-blocks.md](ui/provider-blocks.md) and [provider-block-content.md](ui/provider-block-content.md); the Popover only trims chrome around them, not the underlying data.

Two decisions are now fixed:

- **The Popover shows exactly what the Main Window cards show.** Same Limits/Subscription content, same values, not a reduced-information variant. The one confirmed content difference is the per-card update-frequency dropdown, dropped from the Popover card — see [Card Content](#card-content).
- **Display settings apply synchronously between Main Window and Popover.** Show limits / Show plan / Show source / Show update time ([settings.md#display](ui/settings.md#display)) reflect the same state in both surfaces. The synchronization mechanism is now implemented: `settings.js` emits a Tauri app event (`SETTINGS_CHANGED_EVENT`, `"settings-changed"`, defined in [constants.js](../../frontend/modules/constants.js)) after every settings save, over `window.__TAURI__.event.emit`; `popover.js` listens for it via `window.__TAURI__.event.listen` and re-derives its own state from the (freshly reloaded) settings model. This also covers provider-visibility (Cursor/Claude/Codex) settings for the Popover's tab list — see [Toolbar](#toolbar) and [View Tabs](#view-tabs) for the details and what is/isn't covered.

## Architecture

- **Application** — the whole app process.
- **Main Window** — the existing window described throughout [architecture.md](architecture.md) and [ui/](ui).
- **Menu Bar Popover** — a popover opened from the menu bar icon.

The Popover is not a separate application: same process, same [frontend modules](architecture.md#frontend-modules), same Tauri commands, same [settings](ui/settings.md).

## Roles

**Main Window**
- onboarding
- settings
- complex scenarios
- help

**Menu Bar Popover**
- quick glance
- quick actions
- brief information

## Card Content

Popover cards drop one element present on the Main Window card: the per-card update-frequency dropdown button (the settings-gear-styled trigger in the card header that opens the frequency menu and manual "UPDATE NOW", documented in [provider-blocks.md](ui/provider-blocks.md)). Everything else in the card — Limits section, Subscription section, source line — is unchanged, per the "shows exactly what the Main Window cards show" decision above.

Per-provider manual refresh and frequency control move to the toolbar instead (see below); they are not duplicated per card in the Popover.

## Toolbar

The panel's own chrome is two fixed strips with the scrolling card area between them:

```text
 AI Limits                                    [update all]
 [   All   |  Codex  |  Claude  |  Cursor   ]
 ─────────────────── cards ───────────────────
 Open Application                     [info] [gear]
```

- **`[update all]`** — refreshes every visible provider. A hand-authored circular refresh glyph (`POPOVER_UPDATE_ALL_ICON_SVG` in [popover-toolbar.js](../../frontend/modules/popover-toolbar.js)), drawn at the panel's own icon scale (15px, 1.5 stroke, `currentColor`) rather than reused from the Main Window's 18-20px/2.0 glyph builders.
- **`[All] [Codex] [Claude] [Cursor]`** — a macOS segmented control, one segment active at a time, switching what the card area below shows (see [View Tabs](#view-tabs)).
- **`[info]`** — opens the Main Window, on the matching Help/info section.
- **`[gear]`** — opens the Main Window, on Settings.

`[info]` and `[gear]` sit in the footer, not next to `[update all]`. They are navigation *out* of the panel, and system panels put that kind of row at the bottom ("Wi-Fi Settings…", "Sound Settings…"); keeping them out of the top strip also leaves it to the two things that act on the content directly below it. Neither opens any UI inside the Popover itself — both are pure navigation into the Main Window, consistent with [Roles](#roles): settings and help stay Main-Window-only.

**Implemented.** `[info]`, `[gear]`, and the "Open Application" row (see [Entry Points](#entry-points)) are real navigation, not stubs. `popover.js` defines `window.__openMainWindowHelp(chapterId?)`, `window.__openMainWindowSettings()`, and `window.__openMainApplication()` — the same three bridge-global names the earlier frontend-only phase left as documented no-ops — as thin wrappers around `window.__TAURI__.core.invoke(...)`, calling three Tauri commands in [src-tauri/src/commands/mod.rs](../../src-tauri/src/commands/mod.rs):

- `open_main_window_help(chapter: Option<String>)` — shows+focuses `"main"`, then evals `window.__openHelpFromNative && window.__openHelpFromNative(<chapter or nothing>)` on it. Passing no argument at all (rather than an explicit `null`) when `chapter` is absent matters: `main.js`'s `openHelp(chapterId = DEFAULT_HELP_CHAPTER)` only applies its default for `undefined`, not `null`.
- `open_main_window_settings()` — shows+focuses `"main"`, then evals `window.__openSettingsFromNative && window.__openSettingsFromNative()`.
- `open_main_window()` — shows+focuses `"main"`, no eval (just brings the window forward, for "Open Application").

All three mirror the pattern `handle_menu_event` already used for the native Help/Settings menu items (same eval calls, same `Option`-guarded `get_webview_window("main")` lookup, no-op rather than error if `"main"` doesn't exist), just triggered by a Popover click and an `invoke()` instead of a native menu event. Each also explicitly hides the `"popover"` window on macOS after focusing `"main"`, rather than relying solely on the Popover's existing `WindowEvent::Focused(false)` handler (which *does* also fire in this case) — the explicit `.hide()` removes any dependency on that handler's exact timing relative to `set_focus()`, and is a harmless no-op if the other handler already hid it first. These commands are not macOS-`cfg`-gated themselves (the popover-hide step inside them is); calling them on another platform just shows/focuses `"main"`, which is harmless.

The two strips are built by [popover-toolbar.js](../../frontend/modules/popover-toolbar.js) — `buildPopoverToolbarHtml()` and `buildPopoverFooterHtml()` — and shared verbatim with the `?showcase=popover` preview. Their handlers are attached separately (`attachPopoverToolbarHandlers` / `attachPopoverFooterHandlers`) because only the top strip is re-rendered when the tab list changes; attaching the footer's handlers on every rebuild would stack duplicates on an element that was never replaced.

## View Tabs

- **`All`** — every visible provider's card, stacked top to bottom. This is the view shown when the Popover opens.
- **`Codex` / `Claude` / `Cursor`** — selecting one shows only that provider's card, hiding the others.
- The per-provider tab list follows the same [provider visibility settings](ui/settings.md) as the Main Window (Cursor / Claude / Codex toggles): a disabled provider's tab does not appear. `All` always appears regardless of how many provider tabs are currently shown.
- The tabs are a segmented control, not a row of buttons: an alpha-filled track holding equal-width segments, the selected one a raised pill. Accessibility follows the role rather than the look — each segment is `role="tab"` inside the `role="tablist"`, and selection is carried by `aria-selected`, not the button-flavored `aria-pressed` an earlier pass used. The track never scrolls horizontally; four segments always fit the panel's fixed width.
- **Live sync, implemented.** The tab list is no longer a one-time snapshot taken at window load. `popover.js` listens for the `SETTINGS_CHANGED_EVENT` Tauri event (see [Intent](#intent)) and rebuilds the toolbar/tab list from the freshly-reloaded provider-visibility settings whenever it fires — so toggling Cursor/Claude/Codex in the Main Window (the Main Window's Providers section labels the Claude toggle "Cloud", and settings.js still stores it under the `cloud` key; the provider id everywhere else is `claude`) while the Popover is already open updates its tab list immediately, without reopening the Popover. If the currently-selected tab's provider is the one that just got disabled, the selection falls back to `All`; otherwise the current selection (including a specific provider tab) is preserved across the rebuild.

## Cross-Window Sync

Each Tauri webview window runs its own JS context with its own module-scoped copy of `settings.js`'s `appSettings` and `theme.js`'s `appTheme`, so a change saved in the Main Window updates localStorage but not an already-open Popover. Two Tauri app events bridge that gap, both defined in [constants.js](../../frontend/modules/constants.js) and both emitted only under Tauri (a plain-browser context has no other window to notify):

- `SETTINGS_CHANGED_EVENT` (`"settings-changed"`, payload `{ kind: "visibility" | "display" }`) — emitted by `settings.js` after every settings save (`handleSettingsChange` / `handleDisplaySettingsChange`). `popover.js` listens, calls `reloadAppSettings()`, then re-derives everything from the fresh state: it rebuilds the toolbar/tab list (preserving the current tab selection if still valid, per [View Tabs](#view-tabs)) and calls `refreshProviderSectionsFromCache()` to re-render the mounted cards' Limits/Plan/Source/update-time sections. The handler does not branch on `kind` — both kinds are cheap and idempotent to redo, so it does both unconditionally. Covers provider visibility (Cursor/Claude/Codex) and all four display toggles; no other settings need cross-window sync (notifications and auto-update are Main-Window-only, and the Popover doesn't read them).
- `THEME_CHANGED_EVENT` (`"theme-changed"`) — emitted by `theme.js`'s `setManualTheme()`, i.e. the Main Window's "Dark theme" toggle. `popover.js` listens and calls the new `reloadAppTheme()`, which re-reads the theme storage key into this window's `appTheme` and re-applies `data-theme`. Kept separate from the settings event because the theme lives in its own storage key and its own module, and reacting to it means re-applying an attribute rather than re-rendering anything. *System* theme changes need no event at all: every window observes the `prefers-color-scheme` media query itself, which already worked.

The Popover has no settings UI of its own, so it is only ever a listener, never an emitter, and `initSettings({})` is called there without `onChanged`/`onDisplayChanged` callbacks — those fire only for a change made *within* the same window, which cannot happen here.

## Entry Points

- **Menu bar icon** — a left click opens/closes the Popover; behaves as a toggle. It does not open or activate the Main Window. Implemented: a `TrayIcon` built in `install_tray_icon` ([src-tauri/src/main.rs](../../src-tauri/src/main.rs)) handles left `TrayIconEvent::Click`, toggling the `"popover"` window's visibility and never touching `"main"`. A right click opens a small context menu instead — see [Menu Bar Icon](#menu-bar-icon).
- **Dock icon** — opens or activates the Main Window. It does not open the Popover. Implemented via `tauri::RunEvent::Reopen` (the NSApplicationDelegate `applicationShouldHandleReopen` callback, surfaced through `App::run`'s event callback rather than `.setup()`) — on reopen, `install_main_window_close_guard`'s window is shown and focused; the Popover window is never referenced by this handler.
- **"Open Application" button inside the Popover** — opens the Main Window. Implemented: `popover.js`'s `window.__openMainApplication()` invokes the `open_main_window` Tauri command — see [Toolbar](#toolbar) for the full implementation writeup (shared with `[info]`/`[gear]`).

## Closing

- Closing the Popover hides only the Popover. Implemented: the Popover window's `WindowEvent::Focused(false)` (loses focus / user clicks elsewhere) calls `.hide()`, never `.close()`.
- Esc closes the Popover, the way a system popover does. `popover.js` registers a `keydown` listener that invokes the `hide_popover` Tauri command. It is registered in the *capture* phase so it runs before providers.js's own bubbling Escape handler, which closes a card's open dropdown: if a dropdown is open, Esc belongs to that dropdown and the window stays up. Outside Tauri (the page opened in a plain browser) `window.__TAURI__` is absent and the handler is a no-op.
- Closing the Main Window closes only that window; its Dock icon disappears if not pinned; the application keeps running (menu bar icon stays, Popover remains available). Implemented: `install_main_window_close_guard` intercepts the Main Window's `WindowEvent::CloseRequested` (red traffic light, Cmd+W), calls `api.prevent_close()`, and hides the window instead of letting it destroy.
- Quit fully terminates the application. Verified unaffected by the above: the native Quit menu item (`PredefinedMenuItem::quit` in `install_help_menu`) terminates the app directly through the OS's own menu action, not through a window's close-request path, so the close-guard never intercepts it.

## Launch Behavior

- **First launch** — open the Main Window. No change needed: this is default Tauri window-config behavior (`app.windows` in tauri.conf.json), confirmed still true — the Popover window is built hidden (`.visible(false)`) in `install_popover_window` and never shown at startup.
- **Normal manual launch** — open the Main Window. Same as above — unchanged, default behavior, confirmed still holds.
- **Autostart/login item** — launch in the background, show the menu bar icon, do not open the Main Window, do not open the Popover automatically. **N/A for this phase**: this app has no launch-at-login / autostart feature at all yet (no `tauri-plugin-autostart`, no LaunchAgent, no login-item registration anywhere in the codebase). This case is blocked on a launch-at-login feature that doesn't exist yet — implementing it is a separate, larger feature outside this phase's scope. Tracked in [Open Items](#open-items).

## UX Principle

- Menu Bar = quick access.
- Main Window = full work.
- The Popover opens only on an explicit user action (clicking the menu bar icon) — never automatically.

## Static Layout

Approach: build the Popover's real markup and styles first, previewed as static layout through the browser showcase, before any native tray/window logic exists. Once the layout is settled, the macOS-side logic (menu bar icon, popover open/close, window wiring) is built around it.

Two things now exist side by side:

- **The browser showcase preview** — [showcase.js](../../frontend/modules/showcase.js) / [showcase.css](../../frontend/styles/showcase.css), reachable via `?showcase=popover`, driven by mock `SHOWCASE_PROVIDERS` data.
- **The real, standalone Popover surface** — [popover.html](../../frontend/popover.html) / [popover.js](../../frontend/modules/popover.js), a second static entry point alongside [index.html](../../frontend/index.html) / [main.js](../../frontend/modules/main.js). It sits at the same level as the Main Window's entry point rather than living inside it, matching the "not a replacement, a different surface" framing in [Intent](#intent). It is now reachable in the built app: `install_popover_window` in [src-tauri/src/main.rs](../../src-tauri/src/main.rs) points a `WebviewWindow` labeled `"popover"` at `popover.html`, the same way the existing `"main"` window is pointed at `index.html` via `frontendDist`. It is real in every other sense too: it drives its own provider cards through the same [providers.js](../../frontend/modules/providers.js) pipeline the Main Window uses (`initProviders`, `refreshEnabledProviders`, `initProviderIntervals`), against the real `get_single_provider_limits` Tauri command and the real settings/localStorage model — not mock data. Loaded in a plain browser (no `window.__TAURI__`), it shows the same "Tauri API is unavailable" status-line error the Main Window would show under the same condition; that's expected, not a bug.

Both surfaces share one module, [popover-toolbar.js](../../frontend/modules/popover-toolbar.js), for all [Toolbar](#toolbar)/[View Tabs](#view-tabs) markup and behavior (icons, tab markup, the tab-filter function, toolbar-button wiring), so they cannot drift apart. `showcase.js` no longer has its own inline copy of this logic.

Implemented, in both surfaces unless noted:

- `?showcase=popover` remains a fourth showcase mode alongside `macos`, `windows`, `linux`. It still does not render a duplicate or hand-copied mockup: it relocates the same live `#provider-list` element that `initProviders()` already mounts inside `.app` — the same real card markup, the same [`SHOWCASE_PROVIDERS`](../../frontend/modules/showcase.js) mock data used by the other three platforms, the same interactivity — into the same `.popover-root` panel structure `popover.html` uses. `popover.html` mounts its own, separate `#provider-list` of the same shape, fed by the real pipeline instead.
- Unlike the `macos`/`windows`/`linux` showcase variants, the panel has no titlebar and no traffic lights — the Popover is not an OS window, so that chrome does not apply. It also has no resize handle; the OS-window screenshot resize behavior in [showcase.md](../product/showcase.md) does not apply to it either.
- No Help entry point.
- The [Toolbar](#toolbar) and [View Tabs](#view-tabs) are implemented, `All` selected by default, clicking a provider segment filters the stacked cards down to that one card.
  - In `popover.html`, `[update all]` is real, local behavior: it calls `refreshEnabledProviders()` for this window's own mounted cards, no cross-window IPC involved.
  - `[info]` and `[gear]` are real cross-window navigation: they call two bridge globals, `window.__openMainWindowHelp(chapterId?)` and `window.__openMainWindowSettings()`, implemented via `window.__TAURI__.core.invoke` calls into the `open_main_window_help`/`open_main_window_settings` Tauri commands — see [Toolbar](#toolbar). A provider card's own "More details" link (shown on some data-error states) also routes through `window.__openMainWindowHelp`, via a small guard in [help.js](../../frontend/modules/help.js)'s `openHelp()`: when no local Help view exists (true for `popover.js`, which never calls `initHelp`), it defers to that same bridge global instead of throwing.
  - The "Open Application" footer row calls a third bridge global, `window.__openMainApplication()`, same pattern, backed by the `open_main_window` command.
  - In the showcase preview the four actions are rendered-only, with no callbacks attached — the preview does not go through `popover.js` at all. Tab switching does work there, since that behavior lives in the shared module.
- The [Card Content](#card-content) decision is implemented the same way in both surfaces: the per-card update-frequency dropdown (`.provider-settings-menu`) is hidden with a scoped CSS rule (`.popover-root .provider-settings-menu { display: none; }`) rather than a render-time option on `renderProvider()`. Three cards is not enough to justify threading a new option through `provider-rendering.js` and every caller, and the dropdown's underlying DOM/handlers being present-but-hidden costs nothing observable. Revisit if `renderProvider()` grows more Popover-specific variations later.
- The provider tab list follows the Main Window's provider-visibility settings (Cursor/Claude/Codex toggles) in `popover.html`: `PROVIDER_IDS.filter(isProviderEnabled)` is passed into `buildPopoverToolbarHtml()`, so a disabled provider's tab does not appear, per [View Tabs](#view-tabs). This is a live subscription, not a one-time snapshot — see [Cross-Window Sync](#cross-window-sync). The showcase preview always shows all three tabs, since all three `SHOWCASE_PROVIDERS` are unconditionally "enabled" there.
- Stylesheets: `popover.html` links its own entry point, [frontend/popover.css](../../frontend/popover.css), instead of the Main Window's `styles.css`. It pulls in only what this surface renders — tokens, the shared base, the provider cards, and the panel layer — leaving out the Main Window toolbar, the settings dropdown, the help view, the update banner and the screenshot-showcase chrome, none of which exist here. The panel layer itself is [frontend/styles/popover.css](../../frontend/styles/popover.css), imported last by both entry points so its scoped overrides win; it is a real file of its own rather than a section of `showcase.css`, so the shipped window no longer depends on the screenshot-mockup stylesheet. See [Visual Layer](#visual-layer).
- CSP: `popover.html` introduces no new script/style/resource origins, so it is covered by the same `app.security.csp` in `src-tauri/tauri.conf.json` as `index.html` without changes — `script-src 'self'`, `style-src 'self'`, no inline styles or scripts, only local ES module imports.

Native tray/window logic now exists on top of this layout — see [src-tauri/src/main.rs](../../src-tauri/src/main.rs): a `TrayIcon` (`install_tray_icon`) toggles the `"popover"` `WebviewWindow` (`install_popover_window`), positioned near the clicked tray icon (`position_popover_near_tray`) and hidden on focus loss. Dock-icon reactivation and Main Window close-to-hide are wired up too (see [Entry Points](#entry-points) and [Closing](#closing)). `[update all]`'s real behavior still stops at this window's own cards (no cross-window refresh — this was never in scope for the native phase). `[info]`/`[gear]`/"Open Application" now call real implementations of the bridge globals, backed by new Tauri commands in [src-tauri/src/commands/mod.rs](../../src-tauri/src/commands/mod.rs) — see [Toolbar](#toolbar) for the full writeup.

## Visual Layer

The panel is styled to read as a macOS menu bar panel (Control Center, the Wi-Fi/Sound/Battery panels), not as a scaled-down Main Window. Everything below lives in [frontend/styles/popover.css](../../frontend/styles/popover.css), scoped under `.popover-root` so none of it leaks into the Main Window.

**The panel owns no size of its own.** Width comes from the native window (`POPOVER_WIDTH` in [src-tauri/src/windows.rs](../../src-tauri/src/windows.rs)) and the panel simply fills it at `width: 100%`, so that number lives in exactly one place; height is `100vh` for the same reason (see [Window Size](#window-size)). The one geometric constant the panel does restate is the corner radius — see below.

**No card inside the window.** An earlier pass drew a `.popover-surface` box with an 88%-opaque background, a 1px border and a CSS shadow, inset by a 10px body padding — which covered the native vibrancy almost entirely, leaving the blur visible only as a thin frame. There is no such box now: the webview paints no panel background, no panel border and no panel shadow. Content runs edge to edge with ~10px side padding and slightly more top/bottom, so the native material is what fills the panel. `.popover-root` carries `border-radius: 11px` and `overflow: hidden` — the same 11pt the native window is given (`radius: Some(11.0)` in main.rs), so hover highlights and the footer hairline clip on the window's own curve. The `--mix-popover-surface` token the old model needed is gone from `tokens.css`.

**Everything is an alpha overlay.** Nothing inside the panel paints an opaque color over the material: text is near-opaque black/white, surfaces are white- or black-alpha washes. The panel defines its own `--pop-*` variables and, in the same scope, re-points the app-wide tokens the shared card CSS reads (`--text`, `--text-muted`, `--surface`, `--meter-bg`, the font sizes) at them. The global palette is untouched.

**Typography.** The Main Window's `--font-family-base` leads with Inter, for which this repo ships no `@font-face` — it resolves to whatever the user happens to have installed. A system panel must be San Francisco, so the panel scope uses `-apple-system, BlinkMacSystemFont, system-ui, sans-serif` with `-webkit-font-smoothing: antialiased`. The scale is the system-panel one — 13px body, 11px secondary/captions, 15px ceiling for emphasis — against the Main Window's 13/14/15/20/22, and rows sit at ~20-22px rather than desktop-card spacing.

**Cards as Control Center tiles.** The card keeps its content model unchanged ([Card Content](#card-content)) and changes only its dressing and density. Dressing: no border, no shadow, a 9px radius, and a neutral white-alpha fill — deliberately *the same* fill for every provider. A system panel's modules are neutral and let color carry data (the meters), not identity; the provider is named by its own row heading, so the brand tint an earlier pass put behind each card (and the brand-tinted text tokens the shared card CSS derives) is flattened away here. Meters are 3px, fully rounded.

**Card density.** The Main Window's card layout is what made the panel read as a shrunken app window even after the chrome was fixed, so the tile re-lays the same data in system idioms. All of it is CSS scoped to `.popover-root`; the only change to shared markup is that `provider-rendering.js` now emits key-value text as `label` / `separator` / `value` spans instead of one flat string, which concatenates back to the exact same line in the Main Window.

- **Section heading rules are gone.** `——— LIMITS ———` / `——— PLAN ———` costs two lines per section and is the strongest desktop-card signal inside the tile. The two sections are then handled differently, because they carry different weight: the Limits heading is dropped entirely (its rows are self-describing — a period, a percentage and a meter), while the Subscription heading becomes the *left-hand key* of a key-value row with the plan lines as its right-hand value, costing no extra line at all.
- **Limit rows** are two lines instead of three: the period on the left, the value hard right, the reset time as secondary text between them, and the meter underneath. `.limit-top` is `display: contents` in this scope so its label/value spans join the row's grid directly and can be placed on either side of the reset time, which is a sibling element rather than one of its children.
- **Available credits / Available resets / Plan** are label-left, value-right menu rows, not `Label: value` run together in one column.
- One Main-Window behavior is switched off: `providers.js`'s `applySectionSlotAlignment` writes an inline `min-height` on each section so equal sections line up across side-by-side cards; in a single stacked column that only pads each card with dead space, so the panel overrides it (the one `!important` in the file — an inline style cannot be beaten otherwise — leaving the alignment logic itself untouched for the Main Window).

Measured against the `SHOWCASE_PROVIDERS` mock data, all three provider tiles together occupy ~400px of height, against ~1250px before this pass.

**Controls.** The view tabs are a segmented control (alpha track, raised pill for the selected segment, 12px medium, ~22px tall, 7px/6px radii). Icon buttons are 22x22 with no border or background at rest, a rounded alpha wash on hover and a denser one while pressed. The footer is a full-width hairline with a system-menu row under it, labeled "Open AI Limits" — system panels name the app they open ("Wi-Fi Settings…"), not "the application". Its hover is a neutral alpha wash rather than the system blue selection fill, which would fight the app's own warm/green accent palette.

**No web artifacts.** Text selection, drag-out, tap highlight and the pointer cursor are all off; `popover.js` cancels the `contextmenu` event so right-clicking gives no WebKit Reload/Inspect menu. Focus rings appear only for `:focus-visible` and are a quiet 1.5px inset ring, not base.css's 2px green `--focus` outline. Only the card area scrolls — the top bar and footer are fixed — with `overscroll-behavior: none` (no rubber-band) and a thin overlay scrollbar that is invisible until the pointer is over the list. When the list does overflow, its bottom edge is faded out with a mask so the row the footer hairline cuts through reads as "there is more below" rather than as a layout bug; the fade is toggled by `observePopoverScroll` (popover-toolbar.js, shared with the preview) and is absent whenever the content fits or the list is scrolled to its end.

## Window Size

The width is owned by the native side (`POPOVER_WIDTH`, 348 logical px) and never changes; the panel lays out against the same number.

The height is content-driven, the way system menu bar panels are, rather than a fixed size that is too tall for one provider card and too short for three failing ones. `popover.js` reports the height the panel wants through the `set_popover_height` command: the two fixed strips (top bar, footer) plus the natural height of the card list. It measures the card list, never `.popover-scroll` — that element is stretched to whatever height the window currently has, so measuring it would be circular and would pin the window at its first size forever. The report is driven by a `ResizeObserver` on the card list rather than hooked into each of the many things that change its height (data resolving, a tab filtering cards out, a display toggle syncing in from the Main Window).

The native side clamps the reported value to `[POPOVER_MIN_HEIGHT, POPOVER_MAX_HEIGHT]` (180-620 logical px) and grows the window downwards from its anchor under the tray icon, so a resize never needs a reposition. When the clamp bites, the panel simply scrolls its card area internally — which is why `.popover-root` is `100vh` tall: it always fills whatever the window ended up being, and the scroll area absorbs the difference.

The rest of the native half of the contract (`set_popover_height` in [src-tauri/src/commands/mod.rs](../../src-tauri/src/commands/mod.rs)):

- `height` is the desired **outer** height of the window in logical pixels (CSS px), including the panel's own top/bottom padding. Width is never passed and never changes.
- Out-of-range values are clamped, not rejected — an over-tall panel scrolls rather than running off the screen. `POPOVER_MAX_HEIGHT` (620) is deliberately small enough that the panel still fits below the menu bar on the shortest display Apple ships (1280×800), which is why no additional screen-height clamp is needed at positioning time.
- A non-finite `height` (`NaN`/`Infinity`) is rejected with an error and changes nothing.
- Calling it repeatedly is cheap and idempotent, and calling it while the window is hidden is fine — in fact the normal case, since the frontend loads at startup and reports long before the user first clicks the tray icon.
- On a non-macOS build there is no Popover window and the command is a silent no-op rather than an error.
- `POPOVER_DEFAULT_HEIGHT` (480) is only the height the window is *created* with, before the frontend has reported anything.

All of these constants live in [src-tauri/src/windows.rs](../../src-tauri/src/windows.rs), the single place the window labels and Popover geometry are defined for both `main.rs` (which builds the windows) and `commands` (which looks them up again) — they used to be duplicated in the two modules with a "keep both in sync" comment.

## Native Look and Space Behavior

Two native fixes on top of the plain borderless window described in [Static Layout](#static-layout), both in `install_popover_window` in [src-tauri/src/main.rs](../../src-tauri/src/main.rs):

**Vibrancy/blur material.** The Popover window is built with `.transparent(true)` and `.effects(WindowEffectsConfig { effects: vec![Effect::Popover], .. })`, using Tauri's own built-in window-effects wiring (`tauri::window::Effect`, `tauri::utils::config::WindowEffectsConfig`) rather than hand-rolled `NSVisualEffectView` construction. Tauri already depends on the `window-vibrancy` crate internally for exactly this (see `apply_effects` in that crate's own `src/vibrancy/macos.rs`), so this reuses that existing, version-matched wiring instead of adding a second copy of the same functionality. `Effect::Popover` maps to AppKit's `NSVisualEffectMaterial::Popover` — the same material a real system popover (e.g. a menu bar item's dropdown) uses. This gets the Popover much closer to a real system popover's *look*, but it is still not a real `NSPopover`: there is no native arrow/tip pointing back at the tray icon, and building one would mean wrapping the window in an actual `NSPopover` (a bigger, separate task — tracked in [Open Items](#open-items)).

For the vibrancy to actually show through, nothing in the webview may paint an opaque background: `body.popover-page` is `transparent`, and the panel content is built entirely from alpha overlays with no opaque container of its own — see [Visual Layer](#visual-layer). The `?showcase=popover` preview has no real vibrancy (it is not a native window), so its stage supplies a colored backdrop and a stand-in shadow around the panel; the panel itself is styled identically in both places.

**`macOSPrivateApi` opt-in.** `.transparent(true)`/`.effects(...)` on macOS require Tauri's `macos-private-api` Cargo feature (added to the `tauri` dependency features in `src-tauri/Cargo.toml`) and the matching `"macOSPrivateApi": true` flag under `app` in `src-tauri/tauri.conf.json` — Tauri's own build script (`tauri-build`) hard-errors at compile time if the two are out of sync, so `cargo build` is itself the check that they agree. This is a deliberate, documented tradeoff worth calling out explicitly: it opts the app into undocumented/private macOS APIs (per Tauri's own description, currently the transparent-window functionality and setting `fullScreenEnabled` to `true`) rather than only Apple's public, App-Store-safe surface. It's standard, common practice for Tauri apps that want any kind of vibrancy/blur effect — there is no public-API path to it in Tauri today — but it is a real opt-in, not a no-op default, and should be kept in mind for anything that later cares about App Store distribution or relies on Apple's public-API guarantees.

**Space-switching fix.** A plain `NSWindow`'s default `collectionBehavior` does not include `NSWindowCollectionBehaviorCanJoinAllSpaces`, so ordering it to the front (which is what Tauri's `show()`/`set_focus()` do under the hood) switches the user back to whichever Space the window "belongs to" — the same Space the Main Window lives on. `install_popover_window` now calls a new `set_collection_behavior` helper right after building the window, which reaches into the window's native `NSWindow` via `WebviewWindow::ns_window()` (a `*mut c_void` Tauri already exposes for exactly this kind of native customization) and sets its `collectionBehavior` via `objc2`/`objc2-app-kit` — the same crate family (and the exact same resolved versions, `objc2` 0.6.4 / `objc2-app-kit` 0.3.2) Tauri's own macOS backend already uses internally, added as direct `[target.'cfg(target_os = "macos")'.dependencies]` in `src-tauri/Cargo.toml` rather than pulling in a second, older `cocoa`/`objc`-family crate.

Flags set, combined via `NSWindowCollectionBehavior`'s bitflags: `CanJoinAllSpaces` (the actual fix — the window is treated as part of every Space, so showing it never triggers a Space switch), `Stationary` (the window doesn't get dragged along by Space-switching gestures — it's a fixed menu-bar accessory, not a document window), `IgnoresCycle` (excluded from Cmd+`~`-style window cycling and the Window menu's window list, matching the existing `skip_taskbar(true)`), and `FullScreenAuxiliary` (so the Popover can still be reached as an auxiliary window while some other app occupies a fullscreen Space, the way the menu bar itself remains reachable). `Transient`/`Managed` and `CanJoinAllApplications` were deliberately left out — see the doc comment on `set_collection_behavior` in main.rs for why.

**What could not be verified directly:** none of this — the vibrancy material, the Space behavior, or general window appearance — could be visually verified in the sandbox this was built in (no display available). It was verified by reading the actual resolved crate sources instead: Tauri 2.11.5's `WebviewWindow::ns_window()` implementation and doc example (`src/window/mod.rs`, `src/webview/webview_window.rs`), its internal `window-vibrancy` wiring (`src/vibrancy/macos.rs`), and `objc2-app-kit` 0.3.2's generated `NSWindow`/`NSWindowCollectionBehavior` bindings — and by confirming `cargo check`, `cargo clippy`, and `cargo build` all succeed cleanly against this project's actual dependency graph. It should be checked on a real build on macOS: does the popover visibly blur the desktop/windows behind it, and does opening it while on a different Space actually stay on that Space instead of jumping back.

## Window Frame

Everything here is set in `install_popover_window` in [src-tauri/src/main.rs](../../src-tauri/src/main.rs), on top of the vibrancy and Space behavior described above.

- **Floats above other windows.** `.always_on_top(true)`. A menu bar panel that can end up behind the window it was opened over is not a menu bar panel; Control Center and the Wi-Fi/Sound panels always sit on top. Without this the Popover was an ordinary window in the normal z-order.
- **Native window shadow.** `.shadow(true)` (`NSWindow.hasShadow`, tao's default — stated explicitly because it is load-bearing here) plus an `invalidate_native_shadow` helper called after every `show()`. The second part is the part that matters: a transparent window's shadow is derived from the alpha it actually rendered and AppKit caches that shape, so after the window's content or size changed while hidden — both happen here, since the vibrancy mask is applied to the webview surface and [Window Size](#window-size) resizes the window between showings — the cached shadow can be stale (rectangular behind rounded corners) or missing. `invalidateShadow` discards it. The helper goes through `objc2-app-kit`, the same route and the same safety contract as `set_collection_behavior`. The webview no longer paints a CSS shadow of its own; a CSS shadow could never extend past the window bounds anyway.
- **Corner radius 11.** `radius: Some(11.0)` in the `WindowEffectsConfig` (`POPOVER_CORNER_RADIUS`), matching the `border-radius: 11px` the panel content uses — see [Visual Layer](#visual-layer). The two must stay equal or the content's own clipping will not sit on the window's curve.
- **Not in Cmd+Tab or the Window menu.** Unchanged from before, and worth restating: `skip_taskbar(true)` plus `NSWindowCollectionBehavior::IgnoresCycle`. Note that Cmd+Tab switches *applications*, not windows — AI Limits itself still appears there, as any app with a Dock icon does; what these flags guarantee is that the Popover is not offered as a separate window to cycle to, and does not show up in the Window menu's window list.
- **Activation.** Showing the Popover calls `show()` + `set_focus()`, which activates the application (its menu bar replaces the frontmost app's) — the Popover needs real key-window status for its Esc handler and its buttons. A true system panel takes keyboard focus *without* activating its owner, which requires an `NSPanel` with the non-activating style mask; a Tauri window is an `NSWindow` and cannot be reclassified after creation, so this is a known, accepted difference rather than an oversight.
- **Dismiss on focus loss is unconditional.** `WindowEvent::Focused(false)` → `hide()`, with no filtering. This was reviewed rather than inherited: every focus-loss path this app can actually produce — clicking another app, clicking the Main Window, opening an external URL in the browser, starting a CLI login in Terminal, opening the tray icon's own right-click menu — is one where hiding is the correct behavior. The app presents no native sheets, alerts or file panels of its own that could take focus while the user still means to keep the Popover up, so there is nothing to distinguish. Revisit if such a dialog is ever added.

## Positioning

`position_popover_near_tray` anchors the Popover horizontally centered on the clicked tray icon, `POPOVER_MENU_BAR_GAP` (6pt) below it, and keeps it `POPOVER_SCREEN_MARGIN` (8pt) clear of the display's left, right and bottom edges. Three earlier defects are fixed:

- **The right display.** It used `primary_monitor()`, so on a multi-monitor setup a click on a secondary display's menu bar put the panel on the primary one. It now finds the display containing the icon.
- **Screen edges.** It only did `.max(0.0)`, so an icon near the right edge of the menu bar (the common case — a newly added status item lands at the left of the *existing* ones, but a nearly empty menu bar puts it far right) pushed half the panel off screen. Both edges are clamped now.
- **The menu bar gap.** It placed the panel flush against the bottom of the menu bar; system panels leave a few points.

The coordinate handling is the fiddly part and is worth stating, because three different "physical" conventions meet here:

- `TrayIconEvent::Click`'s `rect` is already in physical pixels — tray-icon's macOS backend takes the status item window's AppKit frame (points, bottom-left origin) and multiplies it by *that window's* backing scale factor, flipping y to a top-left origin. It is not a logical rect, so the old `to_physical(scale_factor)` call on it was a no-op that merely looked like a conversion.
- `Monitor::position()`/`size()` are "physical" in the same sense: each monitor's own logical AppKit bounds times its own scale factor.
- `set_position` with a **logical** position is the only unambiguous target: tao converts it straight into a global top-left-origin AppKit point. A *physical* position would first be divided by whatever scale factor the window sits on **before** the move — exactly wrong when the move is to a differently-scaled display.

So the whole computation is done in logical points, and the display is identified by testing each monitor's own scale factor: dividing the icon rect by scale factor `s` only lands inside the logical bounds of the monitor whose scale factor really is `s`. That makes mixed-DPI setups (Retina laptop plus a 1x external display) fall out correctly instead of relying on a single global scale factor.

## Menu Bar Icon

**Template image.** The tray icon is `icons/icon-tray.png`, a purpose-built macOS template image: monochrome, alpha-only, 18pt rendered at @2x (36×36 px), loaded with `TrayIconBuilder::icon(..)` + `.icon_as_template(true)`. `icon_as_template` sets `NSImage.isTemplate`, which is what makes AppKit tint the shape for the current menu bar appearance — light or dark, and inverted while the icon is highlighted — instead of blitting the pixels verbatim. The previous placeholder was the full-color app bundle icon (`app.default_window_icon()`), which is the single most obvious "this is not a native app" tell in the menu bar.

The artwork is the app's own four-point mark from `icons/icon-master.svg` with the per-arc brand colors stripped and one uniform stroke, generated by `scripts/generate-desktop-icons.sh` alongside the other icon assets: it writes `icons/icon-tray.svg` (24-unit viewBox rendered at 18pt, artwork scaled to 0.80 for optical margin, 2.35 stroke ≈ 1.4pt — the SF Symbols-like weight the menu bar expects) and rasterizes it through `scripts/render-svg-png.mjs`, a small Playwright/Chromium rasterizer. Playwright is already a devDependency for the showcase screenshots, and the Tauri CLI's own `tauri icon` cannot produce this (it only emits the full app-icon set). The master's fifth path — the doubled accent arc — is dropped, since it is invisible once the artwork is monochrome at menu bar size.

36×36 is the only size shipped: the tray API takes exactly one image and rescales it to an 18pt height, and macOS has no @3x displays. The PNG is `include_bytes!`-embedded in the binary rather than read from disk, so it ships inside the `.app` with no resource-path resolution and no missing-file failure mode; `bundle.icon` in tauri.conf.json is unrelated and unchanged. Loading it needs Tauri's `image-png` Cargo feature (added in `src-tauri/Cargo.toml`) for `Image::from_bytes`.

**Right-click menu.** `build_tray_menu` attaches a three-item context menu — "Open AI Limits", "Settings…", separator, "Quit" — shown on right click only (`show_menu_on_left_click(false)` keeps the left click as the Popover toggle). Its items carry the same menu ids the native app menu already uses, and `handle_menu_event` dispatches both by delegating to the existing `open_main_window` / `open_main_window_settings` / `open_main_window_help` commands, so each action has one implementation regardless of entry point. Quit is `PredefinedMenuItem::quit`, the same item the app menu carries.

Two side effects of that consolidation, both improvements: the app menu's own "Settings…" (Cmd+,) and Help items now *show and focus* the Main Window instead of only eval'ing into it — previously they did nothing visible if the Main Window had been closed to the tray — and they hide the Popover, as any navigation into the Main Window does.

## Signing

The Popover is implemented as another window within the existing Tauri application process (for example via Tauri's tray/window APIs), not as a separate native executable, helper app, or app extension (`.appex`).

Consequence: it does **not** require signing beyond the existing pipeline in [macos-signing.md](../devops/macos-signing.md). The Developer ID signature, notarization, and stapling already cover the whole `.app` bundle, including every window and the menu bar icon the same process presents. This is unlike a WidgetKit widget, which is a separate extension target requiring its own nested signature — the Popover has no such separate target.

If a later revision of this design introduces a genuinely separate helper process or extension for the Popover, this section must be revisited; as currently scoped (a window inside the same app), it does not apply.

## Local Testing

Same as the existing Main Window: local unsigned debug builds via `npm run tauri:build:debug` (see [local-build.md](../setup/local-build.md)) are sufficient to test the Popover during development. No additional signing or provisioning step is needed to test it locally, for the same reason given above — it is not a separate signed target.

## Open Items

- **Resolved: `[update all]` icon.** It is now a hand-authored circular refresh glyph drawn at the panel's own icon scale, not a mirrored copy of the Main Window's refresh-cw glyph (which was produced by string-replacing the class attribute inside another module's markup — anything reformatting that module would have silently broken it). See [Toolbar](#toolbar).
- **Resolved: content-driven height.** The window is no longer a fixed `348 × 480`; the height follows the panel's content within a clamped range — see [Window Size](#window-size). The 348pt width is still a fixed decision worth a final confirmation on a real build.
- **Resolved: proper menu bar icon asset.** The tray icon is now a purpose-built monochrome template image generated from the app's master artwork, not the full-color bundle icon — see [Menu Bar Icon](#menu-bar-icon).
- Menu bar icon **state indication** (does the icon change when a limit is nearly exhausted, or when data is stale?) — still open, and deliberately not attempted with the template-image work: a template image is alpha-only, so any state has to be expressed as a shape/badge change rather than a color, which is a design question rather than a plumbing one.
- **Resolved: cross-window settings and theme sync.** See [Cross-Window Sync](#cross-window-sync) — `SETTINGS_CHANGED_EVENT` covers provider visibility and the four display toggles; `THEME_CHANGED_EVENT` covers the manual "Dark theme" toggle, which previously did not reach an already-open Popover at all.
- **Autostart/login item launch behavior — N/A, blocked on a nonexistent feature.** This app has no launch-at-login capability at all yet (no `tauri-plugin-autostart`, no LaunchAgent, no login-item registration anywhere in the codebase). The Launch Behavior spec's "Autostart/login item" case cannot be implemented until that feature exists; building launch-at-login from scratch is a separate, larger feature outside this phase's scope. Revisit this row once autostart exists elsewhere in the app.
- **Resolved: real cross-window navigation for `[info]`/`[gear]`/"Open Application".** See [Toolbar](#toolbar) for the full writeup — `popover.js`'s three bridge globals now invoke `open_main_window_help`/`open_main_window_settings`/`open_main_window`, new Tauri commands in [src-tauri/src/commands/mod.rs](../../src-tauri/src/commands/mod.rs) that mirror `handle_menu_event`'s existing show+focus+eval pattern in main.rs.
- `[update all]`'s real behavior in `popover.js` is scoped to this window's own mounted cards only; there is no cross-window "update all" that also refreshes the Main Window (or vice versa) — each window's provider data is independently fetched and cached in its own JS context.
- **Resolved: positioning across displays and screen edges.** `position_popover_near_tray` now works in logical points, picks the display the tray icon was actually clicked on, clamps both horizontal edges and leaves a gap below the menu bar — see [Positioning](#positioning). Still unverified visually (no display in the sandbox this was built in): it needs a real-build check on a multi-monitor setup, ideally with mixed scale factors, and with the tray icon close to the right edge of the menu bar.
- **Non-activating panel.** Showing the Popover activates the application, because a Tauri window is an `NSWindow` and only an `NSPanel` with the non-activating style mask can take keyboard focus without activating its owner. A real system menu bar panel does not swap the menu bar out from under you. Changing this would mean reclassifying or wrapping the window natively — see [Window Frame](#window-frame).
- **Resolved (with a real caveat): native vibrancy + Space-stable positioning.** See [Native Look and Space Behavior](#native-look-and-space-behavior) for the full writeup — the Popover now applies an `NSVisualEffectMaterial::Popover` vibrancy material and sets `NSWindow.collectionBehavior` so opening it doesn't switch macOS Spaces. Neither could be visually confirmed in this phase (no display in the sandbox); both were verified by reading the actual resolved crate sources and by a clean `cargo build`, but still need a real-build check on macOS.
- **Native frame behavior is unverified on a real build.** Always-on-top, the native `NSWindow` shadow (including the `invalidateShadow` refresh after each show), the 11pt corner radius matching the CSS radius, the content-driven resize and the template tray icon's light/dark/highlight adaptation are all things that only a real macOS build can confirm. They compile and are written against the actual resolved crate sources, but nothing about their *appearance* was verified here — see [Window Frame](#window-frame), [Window Size](#window-size) and [Menu Bar Icon](#menu-bar-icon).
- **No native arrow/tip.** The Popover is still a plain (now vibrancy-backed) rectangular window, not a true `NSPopover` — there is no arrow/tip visually connecting it back to the tray icon it was opened from, unlike a real system popover. Wrapping the window in an actual `NSPopover` (or hand-drawing an arrow shape) is a bigger, separate task that has not been scoped or requested yet.
- **Panel appearance is unverified on a real macOS build.** The [Visual Layer](#visual-layer) was reviewed only through the `?showcase=popover` browser preview, which stands in for the vibrancy material with a flat colored backdrop. How the alpha overlays actually read over a live `NSVisualEffectMaterial::Popover` — tile contrast, hairline visibility, and whether the 11px CSS radius lines up exactly with the native window radius — still needs a look on a real build, in both themes and over both light and dark desktop content.
