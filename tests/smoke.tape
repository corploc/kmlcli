# Smoke test for post-v0.2.0 fixes.
# Generates labelled PNG screenshots at key moments — read these to verify
# the TUI renders correctly, panics don't break the terminal, etc.
#
# Run via: just smoke-test
# Outputs:
#   tests/smoke/*.png  (frames inspected by the agent)
#   tests/smoke/run.gif (full recording, kept for human eyeballing)

Output tests/smoke/run.gif

Set Shell "fish"
Set FontSize 14
Set Width 1200
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

# Build is done by the recipe before vhs runs.
Type "./target/release/kmlcli tests/fixtures/multi.kml" Enter

# Initial render — verifies App::new + install_panic_hook + first prefetch.
Sleep 2s
Screenshot tests/smoke/01-initial.png

# Pan around — verifies handle_action + tile prefetch under MAX_VISIBLE_TILES cap (T4).
Type "lll"
Sleep 500ms
Type "jjj"
Sleep 500ms
Screenshot tests/smoke/02-after-pan.png

# Zoom in — exercises projection.zoom_in + visible_tiles cap.
Type "+++"
Sleep 1s
Screenshot tests/smoke/03-zoomed-in.png

# Zoom way out to world view — exercises low-zoom tile prefetch cap (T4)
# and ll2tile clamp (T9) on very wide viewports.
Type "----------"
Sleep 2s
Screenshot tests/smoke/04-world-view.png

# Toggle the floating tree panel.
Type "t"
Sleep 800ms
Screenshot tests/smoke/05-tree-open.png

# Navigate the tree — exercises tree_visible_rows scroll math (T8).
Down
Sleep 200ms
Down
Sleep 200ms
Down
Sleep 200ms
Enter
Sleep 1s
Screenshot tests/smoke/06-tree-selection.png

# Hide tree, quit cleanly — verifies terminal restore.
Type "t"
Sleep 500ms
Type "q"
Sleep 800ms
Screenshot tests/smoke/07-after-quit.png
