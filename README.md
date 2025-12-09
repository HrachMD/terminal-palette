# terminal-palette

Generate and pick color palettes from the terminal. Think coolors as a TUI app.

<center>
    <img alt="Showcasing terminal-palette" src="demo.gif" width="800" />
</center>

## Installation

```bash
cargo install terminal-palette
```

## Keybinds

### Main view

| Action                           | Key(s)                                   |
| -------------------------------- | ---------------------------------------- |
| Quit                             | <kbd>q</kbd>                             |
| Move selection                   | <kbd>←</kbd> / <kbd>→</kbd>              |
| Add a color block (max 9)        | <kbd>a</kbd>                             |
| Delete selected block (min 3)    | <kbd>d</kbd>                             |
| Open theory selector             | <kbd>x</kbd>                             |
| Open edit-hex dialog             | <kbd>z</kbd>                             |
| Toggle lock on selected block    | <kbd>l</kbd>                             |
| Copy selected block HEX          | <kbd>c</kbd>                             |
| Toggle lock for block N          | <kbd>Alt</kbd>+<kbd>1</kbd>…<kbd>9</kbd> |
| Generate colors (current theory) | <kbd>Space</kbd>                         |

### Theory selector (popup)

| Action         | Key(s)                                       |
| -------------- | -------------------------------------------- |
| Close selector | <kbd>x</kbd> / <kbd>q</kbd> / <kbd>Esc</kbd> |
| Select first   | <kbd>←</kbd>                                 |
| Select last    | <kbd>→</kbd>                                 |
| Move selection | <kbd>↑</kbd> / <kbd>↓</kbd>                  |
| Apply          | <kbd>Enter</kbd> / <kbd>Space</kbd>          |

### Edit color (hex input)

| Action                         | Key(s)                               |
| ------------------------------ | ------------------------------------ |
| Cancel                         | <kbd>z</kbd> / <kbd>q</kbd>          |
| Delete last                    | <kbd>Backspace</kbd>                 |
| Clear field (currently broken) | <kbd>Ctrl</kbd>+<kbd>Backspace</kbd> |
| Apply                          | <kbd>Enter</kbd>                     |
