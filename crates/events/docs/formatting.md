# Console Formatting System

## Used Symbols

- `✓`: Success (green)
- `↻`: Update (blue)
- `+`: Addition (green)
- `-`: Removal (red)
- `→`: Process in progress (gray)
- `⚠`: Warning (yellow)

## Colors

Via the `colored` crate:
- `.green()`: Success, new elements
- `.cyan()`: Values, names
- `.blue()`: Updates, modifications
- `.red()`: Removals, errors
- `.yellow()`: Warnings
- `.dimmed()`: Secondary details
- `.white()`: Normal text
- `.bold()`: Titles

## Layout banner

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Header text
  Details line 1
  Details line 2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```
