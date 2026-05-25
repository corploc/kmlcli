# vhs demo.tape -> docs/demo.gif
#
# Prerequisites:
#   yay -S vhs   (or: go install github.com/charmbracelet/vhs@latest)
#   cargo build --release  (so ./target/release/kmlcli exists)
#
# Run:
#   vhs demo.tape

Output docs/demo.gif

Set Shell "fish"
Set FontSize 14
Set Width 1200
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms
Set PlaybackSpeed 1.0

# Build once before recording so cargo noise stays out of the gif
Hide
Type "cargo build --release" Enter
Wait
Show

Type "./target/release/kmlcli tests/fixtures/multi.kml"
Sleep 500ms
Enter

# Let the TUI render
Sleep 2s

# Pan around with hjkl
Type "lll"
Sleep 500ms
Type "jjj"
Sleep 500ms
Type "hh"
Sleep 500ms
Type "kk"
Sleep 800ms

# Zoom in
Type "+++"
Sleep 1.5s

# Zoom way out — world view
Type "----------"
Sleep 2s

# Zoom back to a useful level
Type "++++++"
Sleep 1s

# Toggle the floating tree panel
Type "t"
Sleep 1.5s

# Navigate the tree and select an element — it should highlight (yellow)
Down
Sleep 400ms
Down
Sleep 400ms
Down
Sleep 400ms
Enter
Sleep 2s

# Pick another element to see the highlight move
Down
Sleep 400ms
Down
Sleep 400ms
Enter
Sleep 2s

# Hide tree
Type "t"
Sleep 1.5s

# Quit
Type "q"
Sleep 500ms
