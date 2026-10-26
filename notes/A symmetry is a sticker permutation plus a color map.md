---
id: 20261024180001
aliases: []
tags: ["type/design", "status/decided", "category/engines"]
---
The 48 cube symmetries are the signed permutation matrices; each is a permutation of the sticker indices of every size, built once from the sticker positions. A state is fixed by a symmetry when some permutation of the six colors t gives s[g(i)] = t(s[i]); t is found by one pass and is unique. The group order is the count of such symmetries and only the identity keeps every color literally. The canonical form renames colors in order of first appearance, which is the smallest renaming, and takes the least of the 48 images; two states are the same pattern up to turning, mirroring and recolouring exactly when the forms are equal, so the library is matched by one lookup per size and the symmetry that carries one onto the other is then found directly to say how they differ.

## Links

- [[A pattern detector reads structure from the cubie state]]
- [[The detector explains patterns and never replaces the search]]
- [[The stabilizer of a state is classified by conjugacy class of subgroup]]

Source: src/patterns/detector/symmetry.rs, lines 1-60
