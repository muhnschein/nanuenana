# Nanünana canvas: Silica style kit (shared by every artboard)

Goal: every artboard looks like a native Sailfish OS 5 Silica app. NO iOS/Android
mannerisms: no tab bars, no bottom nav, no FABs, no hamburger, no toolbars with
titles on the left, no back arrows/buttons, no cards with drop shadows, no
toggle-pill switches, no modal sheets sliding from bottom, no snackbars, no
Material ripples, no checkboxes, no chevrons ">" on list rows, no separators
between list rows, no centred titles.

CSS: `canvas/project/silica.css`, linked from each board. Change it there, never per board.

## Idioms

- PageHeader: right-aligned, light weight, highlight colour; description below it.
- Back navigation = swipe right; shown only by the page-stack indicator (`.ind-back`).
- Actions = PulleyMenu (top, pull down) only. No PushUpMenu. Closed state shows the
  thin glowing `.pull-ind` line. Open: items centred, the one under the finger in
  highlight colour.
- Per-item actions = long-press ContextMenu that expands inline below the item
  (`.cm`). The pressed row gets `.li.pressed`.
- Destructive = RemorseItem in the row.
- Dialogs = full pages with DialogHeader: cancel left (small), accept right (large,
  highlight). No OK/Cancel buttons at bottom.
- Settings controls: TextSwitch (glowing dot left of text), ComboBox (label +
  highlight value), Slider (value above, label below), TextField (underline, label
  below).
- Empty states = ViewPlaceholder (big light highlight text centred + hint).
- Section headers = small, right-aligned, highlight colour.
- Buttons rare, only in page content (`.btn`).

## Nanünana's own parts

- Level = four rising bars (`.lv`), lit up to the level, plus the level's word. Never colour alone.
  Dark ambience: 1 `#ffdd55`, 2 `#ffa94d`, 3 `#ff7a6e`, 4 `#e3a0ff`.
  Light ambience: 1 `#6b5600`, 2 `#9a4a00`, 3 `#b3261e`, 4 `#7b1fa2`.
- Trouble (not updated, no position) = error colour text plus warning triangle, never a level colour.
- Boxed note (`.box`): icon left, small text; for limits and trouble.
- Unofficial line (`.unoff`): smallest text, secondary colour, end of page.
- Issuer's text stays in issuer's language on every board; app's own words in English.
- Owner's rulings: place name = location name, shown once; no custom names; About and
  warning language only in Settings; no footnotes, no subtitle that repeats its control.

## Sizes (Theme, scaled to a 400 px wide phone)

pageMargin 18 · paddingSmall 6 · paddingMedium 12 · paddingLarge 18
itemSizeSmall 56 · itemSizeMedium 72 · itemSizeLarge 90 (header 80)
fontSizeTiny 12 · ExtraSmall 13 · Small 15 · Medium 19 · Large 26 · ExtraLarge 34 · Huge 46
iconSizeSmall 24 · iconSizeMedium 40 · iconSizeLarge 64

Phone frame 400×860 (portrait). A frame may be taller to show a whole scrolled
page; then also set the `$preview` height.

## Board rules

Keep `<script src="./support.js"></script>` exactly; close every element; quote every
attribute; `{{hole}}` is a plain lookup only; all copy is literal markup; no emoji;
icons inline `<svg class="ic" viewBox="0 0 24 24">…</svg>`; real
`<button>`/`<a href>`/`<input>`/`<label>` for controls, `aria-label` on icon-only
buttons; no `<iframe>`; no scripts building DOM.
