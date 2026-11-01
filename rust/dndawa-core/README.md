# dndawa-core

Rust core of **DnDawa_Binary-Magix**: a small turn-based dungeon crawler for the terminal.
One hero, a 5x5 grid of rooms, three monster types, d20 attack rolls, a final boss, and
save/load. No dependencies outside the Rust standard library; `unsafe` is forbidden.

## Build, test, play

```bash
cargo test
cargo run --bin dndawa-crawl -- --seed 5
cargo run --bin dndawa-crawl -- --load saves/run1.sav
```

Commands: `n` `s` `e` `w` move · `a` attack · `wait` · `save <name>` · `help` · `q` quit.
`save run1` writes `saves/run1.sav` (the folder is git-ignored). Plain names only.

## Rules

- Same seed, same dungeon and same rolls (`dice::Rng`, SplitMix64, unbiased).
- Attack: d20 + attack bonus against armor class; natural 1 misses, natural 20 is a
  critical hit (damage dice rolled twice).
- A living foe blocks the way out of its room. Defeat the Bone Warden in the far corner to
  win; reach 0 HP and you lose. A refused action changes nothing, not even the turn.

## Save format

Plain text, version line `DNDAWA-SAVE 2`, then `key=value` lines: `seed`, `turn`, `rng`,
`hero_hp`, `pos`, `rooms` (25 entries, row by row, `-` or `code:hp`), `visited` (25 digits,
`1` = entered). Loading checks every field, the room layout and the visited rooms, and a
loaded game continues exactly like the original. `DNDAWA-SAVE 1` files still load; there,
only the start room and the hero's room count as visited.

## Modules

`dice` · `notation` · `stats` · `combatant` · `attack` · `monsters` · `grid` · `dungeon` ·
`game` · `save` · `board` (the grid with fog) · `play` (terminal text) · `main.rs` (the
`dndawa-crawl` binary).

## Content status

Giant Rat, Skeleton and Goblin statistics come from the SRD 5.1. The hero and the Bone
Warden are homebrew (PENDING_DAWA).

## Legal

This work includes material taken from the System Reference Document 5.1 ("SRD 5.1") by
Wizards of the Coast LLC and available at
https://dnd.wizards.com/resources/systems-reference-document. The SRD 5.1 is licensed
under the Creative Commons Attribution 4.0 International License available at
https://creativecommons.org/licenses/by/4.0/legalcode.
