---
title: "Front-goal request and answer"
description: "The frozen text request and answer of the ABI 3 front-goal export, and what an implementation must reproduce to get the same node counts."
weight: 10
---

# Front-goal request and answer (version 1, frozen)

Normative reference: the Rust source (`src/engines/three/partial/{front,goal,model,groups,ida,twist}.rs`, `src/engines/three/cubie.rs` for CORNER_FACELET and EDGE_FACELET). What follows names what another implementation must reproduce to get the same node counts.

## Input (one TAB-separated text line, 9 fields)

1 version `1` (else status 1) | 2 facelets: 54 letters URFDLB, standard layout U0-8 R9-17 F18-26 D27-35 L36-44 B45-53, centers home
3 twists: six numbers 0-3 for U,R,F,D,L,B, comma separated (sum parity = corner permutation parity, else status 1)
4 accepted: 9 lists separated by space, row-major over front facelets 18..26, each comma separated home facelet numbers 0-53, none empty
5 centerUp 0|1 (1: front center twist must be 0 after the final rotation) | 6 table_ms (0=3000) | 7 search_ms (0=4000)
8 max_nodes (0=5000000) | 9 max_moves (0=21)

## Output: status + text, every text starts with `1`

0 ok: `1 TAB moves TAB rotation TAB nodes TAB tables_ms` (moves standard notation e.g. `R U2 F'`, shortest in HTM; rotation <= 2
  tokens `x y z` with suffix ' or 2, empty = none; tables_ms 3 decimals) | 1 bad input: `1 TAB reason` | 2 no route (none within
  max_moves, or goal unsatisfiable): `1 TAB reason` | 3 search budget (time or nodes): `1 TAB nodes=<n>` | 4 table budget: `1 TAB permille=<n>`

## Reproducing the node counts (tables = Groups9)

Cube model: pieces by cubie model of cubie.rs: corners URF,UFL,ULB,UBR,DFR,DLF,DBL,DRB = 0..7 (facelets CORNER_FACELET), edges
UR,UF,UL,UB,DR,DF,DL,DB,FR,FL,BL,BR = 0..11 (EDGE_FACELET). State per slot: piece, orientation (corner twist 0-2, edge flip 0-1),
and packed center twists. Moves m = 3*face+q, face order U,R,F,D,L,B, q: 0 clockwise, 1 half, 2 counterclockwise (model.rs Turns).
Center twist after move m: face m/3 gains (m%3)+1 quarters mod 4; no other center changes (twist_step).
Goal = views, one per whole-cube rotation r (24, order of rotation_words(), identity first): front position p accepts the stickers of
accepted[map_r[p]]; all other positions accept every sticker; centers must stay home; a view becomes per slot (piece,orientation) bit
masks (goal.rs view_of); drop impossible views, drop duplicates (keep first), drop views whose twist/parity cannot be met. centerUp:
twist mask of front = {0} before rotation, the others free, shifted by the rotation's center add (rotated_twisted). is_met = any view.
Cubie ranking (groups.rs `ranked`): per orbit, score[p] = sum over all views and all constrained slots of 1/(number of pieces the slot
accepts) for each accepted piece p; sort by score descending, ties by lower piece number. This is NOT the research ranking.
Groups9: tracked sets (rank indices into corner ranking C and edge ranking E, groups.rs GROUPS): {C0..C3}, {C4..C7}, {E0..E3},
{E4..E7}, {E8..E11}, {C0,C1,E0,E1}, {C2,C3,E2,E3}, {C0,C1,C2,E0}, {E0,E1,E2,C0}; in this order. Each table: 24^4 byte entries, index =
digit0 + 24*digit1 + 576*digit2 + 13824*digit3 where digit = slot*3+orientation (corner) or slot*2+flip (edge) of the tracked cubie i.
Entry = BFS distance (all 18 moves) from the seed placements: placements met by Acceptance (groups.rs): some view keeps every tracked
cubie allowed in its slot and every slot of that view whose accepted pieces all lie in the tracked set is occupied by a tracked cubie.
Center-twist table (twist.rs): 4096 entries, BFS from vectors some view accepts, present only if any view conditions a twist.
Bound(pos) = max(twist table, the 9 group tables); no corner/edge full tables. Start depth = bound(start).
Search (ida.rs): for depth = bound(start)..=max_moves: dfs(pos, left=depth, last=none). Each dfs entry: nodes++; stop if nodes>max_nodes
(status 3, reports nodes) or (nodes & 0xFFFF)==0 and clock >= deadline (search_ms from search start; table_ms covers the build only).
left==0 -> return is_met(pos). Else for face g in 0..6 (skip g==last, and skip g if last>=3 and g==last-3), q in 0..3, m=3g+q: compute
child bound; skip child if bound >= left; recurse. First leaf found is the answer (no other tie-breaking). Already met start: 0 moves, 0 nodes.
Final rotation = first rotation in rotation_words() order whose own single view is met by the end position (twist rule included), then
shortened to <= 2 tokens by the first match in the order: empty, singles x x2 x' y y2 y' z z2 z', then pairs in that order (effect on the cube).
