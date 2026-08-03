import os

# The macOS app bundle to package. Set by scripts/build-macos-dmg.sh.
app_path = os.environ["DMG_APP_PATH"]
app_name = os.path.basename(app_path)

format = "UDZO"
files = [app_path]
symlinks = {"Applications": "/Applications"}

# Keep in sync with the window layout described in docs/devops/builds.md.
window_rect = ((200, 200), (660, 400))
icon_size = 128
text_size = 16
icon_locations = {
    app_name: (180, 170),
    "Applications": (480, 170),
}
background = os.environ["DMG_BACKGROUND_PATH"]
default_view = "icon-view"
show_icon_preview = False
