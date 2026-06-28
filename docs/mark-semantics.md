# Mark Semantics

This document defines the cursor and block mark semantics used by `ze2` editing commands.

## Coordinate Types

- `logical cursor`: line + grapheme position within the source text.
- `visual cursor`: line + rendered column position on screen.

The distinction matters when a grapheme occupies more than one display cell, such as many CJK characters.

Internally, a `TextMark` stores which coordinate system it uses so later rendering and editing code can interpret the anchors correctly.

## Mark Types

### `mc` - `mark-char`

- Uses the logical cursor.
- The mark range is defined by logical grapheme positions.
- Useful when the selection should follow text structure rather than screen width.

### `ml` - `mark-line`

- Uses the logical cursor.
- The mark range is defined by logical line positions.
- Each selected line is based on text lines, not rendered columns.

### `mb` - `mark-block`

- Uses visual columns.
- The mark range is defined by rendered screen cells.
- This is the most visual mark type.
- Wide characters count as the number of cells they actually occupy on screen.
- `mb` must not be used when soft wrap is enabled.
- Soft-wrapped lines do not have stable visual column boundaries for block editing semantics.

## Mark Kind Compatibility

- A mark can only be extended by the same mark kind that created it.
- If a mark already exists, calling a different mark kind does nothing.
- Example: `ml` followed by `mb` is ignored, and `mb` followed by `ml` is ignored.

## Block Operations

The following commands operate on a block mark and therefore use the block mark's visual-column semantics.

### `mm` - `move-mark`

- Removes the current mark region.
- Saves the removed text to the clipboard.
- Inserts the removed block at the current cursor location.
- For block marks, the moved text is reinserted as a block-shaped visual region.

### `um` - `unmark`

- Clears the current mark.
- Does not modify document contents.
- Does not touch the clipboard.

### `fm` - `fill-mark`

- Replaces the marked region with a repeated fill character.
- For block marks, the fill is applied across the visual block width.
- The width is measured in visual cells, not logical characters.
- Wide characters therefore contribute according to their rendered width.

### `cm` - `copy-mark`

- Copies the current mark contents to the clipboard.
- For block marks, the copied content preserves the visual block shape.
- For line and char marks, the copied content follows logical text ranges.

## Summary Table

| Command | Meaning | Coordinate basis |
| --- | --- | --- |
| `mc` | mark-char | logical |
| `ml` | mark-line | logical |
| `mb` | mark-block | visual |
| `mm` | move-mark | follows mark type |
| `um` | unmark | n/a |
| `fm` | fill-mark | follows mark type |
| `cm` | copy-mark | follows mark type |

## Data-Layer Rule

- `TextMark` stores its coordinate system explicitly.
- All downstream operations must branch on that coordinate system instead of inferring behavior from `kind` alone.
- This applies to `render`, `fill`, `move`, and `copy`.
- If a path touches block marks, it must treat logical and visual marks as different inputs.

## Notes

- "Logical" means text structure and grapheme order.
- "Visual" means rendered terminal cells.
- Block marks should be treated as visual constructs even when they overlap mixed-width text.
- Block marks are only valid when the layout is not soft-wrapped.
