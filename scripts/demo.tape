Output docs/demo.webp

Set Theme "tokyonight"
Set FontSize 22
Set Width 1920
Set Height 1080
Set Padding 25

Sleep 1s
Type "tomo --focus 2m --short-break 1m"
Sleep 1s
Enter
Sleep 1s

# Timer running (auto-started, counting down from 2:00)
Sleep 3s

# Pause
Space
Sleep 1.5s

# Resume
Space
Sleep 1.5s

# Adjust remaining time (+30s x2, -30s x1)
Right Right
Sleep 1s
Left
Sleep 1s

# Show keybindings overlay
Type "/"
Sleep 2.5s
Type "/"

# Task system — open prompt, type name, create
Type "t"
Sleep 0.5s
Type "My Task"
Sleep 0.5s
Enter
Sleep 2s

# Open Settings
Type "s"
Sleep 1.5s

# Edit Focus duration (already selected at TimerFocus)
Enter
Sleep 0.5s
Backspace
Type "3"
Sleep 0.5s
Enter
Sleep 1s

# Navigate down through Timer items to AutoStartOnLaunch, then toggle
Down Down Down Down
Sleep 0.5s
Enter
Sleep 1s

# Quit triggers UnsavedWarning (2 unsaved changes)
Type "q"
Sleep 3s

# Force quit from warning
Type "q"
Sleep 1s
