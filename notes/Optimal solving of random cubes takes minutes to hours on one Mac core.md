---
id: 20261024184501
aliases: []
tags: ["type/evidence", "status/open", "category/engines"]
---
With the Large tables (529 MiB) the optimal search reaches about 5 to 10 million nodes per CPU second on the M2 Pro. Cubes of 8 to 14 moves are proven within a second (30 of 30). Two cubes of 16 moves needed 3.4 x 10^8 and 1.1 x 10^9 nodes (44 s and 124 s); with the Small tables 1.5 x 10^9 and 4.3 x 10^9 (134 s and 341 s), so the large set saves three quarters of the nodes and about two thirds of the time. None of 30 random cubes (about 17 to 18 moves) was proven in 60 s of CPU; the effort grows fourfold to tenfold per move, so the extrapolation is tens of minutes to hours for a random cube. The table size that makes random cubes a matter of seconds is open and belongs to the big hosts (8 edges, 2 bits per state, more databases, several threads).

## Links

- [[The optimal mode bounds its search by corner and edge databases]]
- [[Large hosts measure the big tables, the Mac sets the default]]

Source: docs/engines/3x3.md, lines 111-131
