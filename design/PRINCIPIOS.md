# Spellcaster — interface principles

One page. Every principle carries what it forbids. A review of any screen, widget or theme goes through this list before it turns into code.

Reference person: the operator at the console at 11 p.m. on load-in day, dark room, hand on GO, no time to go looking.

## 1. The graph is the interface

Every widget is a visible node. A Face is a view of the Graph; the Graph lives in the engine and runs the same on the Pi with no screen.

Forbids: a button that does something that is not in the Graph; logic in the GUI; a "magic shortcut" with no matching node.

## 2. One accent, and it means state

Everything idle is gray. Color shows up only when something happens: armed, live, GO, rehearsal, error. If the screen is colored, the show is happening. This comes from the lighting console, not from Material Design.

Forbids: decorative color; accent on an inactive icon; gradient; more than one accent per Theme; the firm's brand color (lime green) as the product accent.

## 3. Console density

8 px grid. Widgets in discrete sizes (1×1 = 48 px, 2×1, 4×4, ...). Nothing elastic, nothing that moves on its own. The operator finds the button with their eyes closed.

Forbids: responsive layout that reflows; rounded corners by default; shadow; layout animation; touch target smaller than 44 px.

## 4. Text first

Command palette with search (Blender's F3). Every command shows the name the CLI and the MCP use. The interface teaches its own vocabulary: whoever learns the GUI already knows how to operate over SSH.

Forbids: an unlabeled icon on a destructive action; a name in the GUI different from the name in the registry; a menu with more than 8 items and no search.

## 5. Themes are looks, Faces are surfaces

The wild part lives in the Theme: window shape via SVG, bezel, LCD, scope, chrome. The functional part lives in the Face: which widgets, where, in which views. One Theme dresses any Face. That is what lets a 2001 Headspace and a 2026 flat operate the same screen.

Forbids: a Theme that changes a widget's position or size; a Face that pins a color; a widget that exists in only one Theme.

## Anti-slop filter (review checklist)

- No decorative gradient. No Material shadow. No glass blur.
- No rounded corners by default (radius 0; 2 px on chips, and that is all).
- No generic library icon in the final screen; a placeholder is accepted if it is marked as one.
- No color without a meaning declared in `tokens/spellcaster.css`.
- No Inter, Roboto, Arial. The product's voice is mono; labels in condensed.
- No emoji. No "✨". No wand, no sparkle, no mystic purple. "Spell" is a written command.
- No generic placeholder text in a mockup: real show, fixture and universe names.
- Every new screen shows, in some corner, the equivalent CLI command.
