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
- [x] Notes persist after closing/reopening the app
  - [x] positions are stored
  - [x] note contents are stored
  - [x] colors are stored
  - [x] sizes are stored
  - [x] consistent
    - need to do a whole application snapshot instead of per note, saves can conflict
- [x] click anywhere in the window and the text focuses
- [x] accelerator to close currently open sticky
- [x] increase autosave frequency
- [x] hide white sticky when loading
- [x] shrink down text size
- [x] fix spacing for headings when nothing is above them
- [x] copy paste
- [x] stickies stick to the edges of other windows
  - this is an issue with macos not the app
- [x] drag and then mouse exiting cursor freezes the task bar
  - fixed as well as possible without changing the menu concept, the cursor change to the resize icons fools the mouseout event, there is not much i can do about this
- [x] increase autosave consistency
  - autosave still fails intermittently but it is no longer an issue
- [x] Can move groups of sticky notes easily
  - drag to select multiple is probably not possible
  - can create an auto organize feature
  - ctrl click to group multiple windows, manipulate position thru os
- [x] cmd+arrows to snap to guidelines
- [x] accelerator to automatically fit window to ~~highest heading level~~ to current size of text
- [x] window height automatically resizes with text
- [ ] item tray icon
  - the one i have now is funny but will get annoying
- [ ] better default colors
- [x] accelerator to iterate through notes
- [ ] notes can sometimes disappear
  - could be a problem with the autosave or with the init process
  - noticed only after switching back to it
  - could have also accidentally hit the accelerator
- [ ] some sort of indicator to show which sticky is currently in focus
