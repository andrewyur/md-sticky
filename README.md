# md-sticky

a sticky note app inspired by the "stickies" app that comes with MacOS. I found it frustrating that it did not support md syntax, and had a ton of unecessary formatting options.

Created with Tauri & Sveltekit

Even though tauri apps are created with cross-platform capabilities, I am creating this app solely for myself and will not make any effort towards cross-platform compatibility.

## Features

- [x] Create new window through window menu
- [x] Can edit sticky note content
- [x] Sticky note content renders as markdown
  - [x] style the checkboxes better
- [x] Notes look like sticky notes
  - no window title
  - custom title bar with no minimize & full screen buttons
  - shaped like a sticky note
- [x] select color ~~through options bar~~
- [ ] Can move groups of sticky notes easily
  - drag to select multiple is probably not possible
  - can create an auto organize feature
  - ctrl click to group multiple windows, manipulate position thru os
- [ ] Notes persist after closing/reopening the app
  - [x] positions are stored
  - [x] note contents are stored
  - [x] colors are stored
  - [x] sizes are stored
  - [ ] consistent
    - need to do a whole application snapshot instead of per note, saves can conflict
- [ ] button to automatically fit window to text
- [ ] window automatically resizes with text
- [x] click anywhere in the window and the text focuses
- [ ] item tray icon
