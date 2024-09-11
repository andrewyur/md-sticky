# md-sticky

a sticky note app inspired by the "stickies" app that comes with MacOS. I found it frustrating that it did not support md syntax, and had a ton of unecessary formatting options.

Created with Tauri & Sveltekit

Even though tauri apps are created with cross-platform capabilities, I am creating this app solely for myself and will not make any effort towards cross-platform compatibility.

## Installation

download the .dmg and run `xattr -d com.apple.quarantine /path/to/dmg.dmg` to allow it to be opened

## Features

- uses a markdown text editor, compatible with gh-markdown syntax (`[ ]` to make checkboxes)
- customizable colors and a large default color palate
- minimal and unobtrusive sticky note appearance
- autosave, notes persist after quitting and reopening the app
- easily move, navigate, resize, and set colors of notes with keyboard shortcuts

## TODO

- [ ] keyboard shortuts to set colors
- [ ] cycle through stickies by height
