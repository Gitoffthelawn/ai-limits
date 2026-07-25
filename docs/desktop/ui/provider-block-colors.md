# Tauri UI Provider Block Accent Color

## Provider Accent Color

Each provider block owns one provider accent token. The token defines the provider-specific brand color used by prominent provider-scoped UI elements.

Provider-specific color values, including theme-tuned variants, should live together in the provider token block. Theme and component selectors should map or consume those tokens rather than scattering provider color values across the stylesheet.

The provider block border and any internal divider that separates provider metadata from provider controls must use the same provider border token stack. This includes the base provider border color and any theme-specific internal overlay/highlight layer used to tune the visible card border. This keeps the divider visible in both light and dark themes and prevents provider-specific overrides, such as Cursor dark theme tuning, from drifting between the card border and its internal separators.

The provider background may use the same provider accent color with lower opacity. Theme-specific overrides may tune opacity or border brightness per provider, but they must do so through the shared provider tokens rather than hard-coding separate divider colors.
