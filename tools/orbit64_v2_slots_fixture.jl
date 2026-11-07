#!/usr/bin/env julia
# Writes tests/fixtures/orbit64_v2_slots.txt from the normative slot tables of flix-orbit64.
#
#     julia tools/orbit64_v2_slots_fixture.jl PATH/TO/flix-orbit64/FORMAT-V2-SLOTS.md \
#         > tests/fixtures/orbit64_v2_slots.txt
#
# Needs only Julia's standard library. Each slot line of the markdown tables becomes one
# line `n|label|facelets` (for example `4|W1 0|1 82`, `5|Z[1,2]|7 11 ...`). The sentence
# "The fixed centers are `face * K + M`" of an odd size becomes `n|fixed|f0 ... f5`.
# The run fails when a size lacks a slot line, has a surplus one, names a slot twice, uses a
# label other than the canonical ones, or does not hold each facelet exactly once.

# The lines of a size: 8 corners, 12 midges and the fixed centers (odd), 24 per wing orbit,
# one per center orbit.
function expected_lines(n)
    odd = isodd(n)
    wings = (n - 2) ÷ 2
    centers = sum(max(0, n - 1 - 2r) for r in 1:(n - 2); init=0)
    8 + (odd ? 12 + 1 : 0) + 24 * wings + centers
end

# The labels of a size: C 0..7, midges M 0..11 and the sentence "fixed" (odd sizes), wings
# W<depth> 0..23 per depth, center orbits Z[r,c] with r <= c <= n-2-r.
function expected_labels(n)
    labels = Set{String}("C $k" for k in 0:7)
    if isodd(n)
        union!(labels, ("M $k" for k in 0:11))
        push!(labels, "fixed")
    end
    for d in 1:(n - 2) ÷ 2, k in 0:23
        push!(labels, "W$d $k")
    end
    for r in 1:(n - 2), c in r:(n - 2 - r)
        push!(labels, "Z[$r,$c]")
    end
    labels
end

function main(path)
    # Collect and check everything first, then print (see orbit64_v2_fixture.jl).
    out = String[]
    sizes = Int[]
    n = 0
    in_block = false
    candidates = 0
    fixed_sizes = 0
    for line in eachline(path)
        m = match(r"^## (\d+)x\1x\1$", line)
        if m !== nothing
            n = parse(Int, m[1])
            push!(sizes, n)
            continue
        end
        m = match(r"^Facelets `0\.\.\d+`\. The fixed centers are `face \* (\d+) \+ (\d+)`\.$", line)
        if m !== nothing
            k, offset = parse(Int, m[1]), parse(Int, m[2])
            push!(out, string(n, "|fixed|", join((f * k + offset for f in 0:5), " ")))
            fixed_sizes += 1
            continue
        end
        startswith(line, "```text") && (in_block = true; continue)
        startswith(line, "```") && (in_block = false; continue)
        in_block || continue
        # Block headings (`corners ...`, `midges`, `wings, depth d`, `centers ...`) start in column 0.
        startswith(line, " ") || continue
        candidates += 1
        m = match(r"^  ((?:[CM]|W\d+) \d+|Z\[\d+,\d+\]): (\d+(?: \d+)*)$", line)
        m === nothing && continue
        push!(out, string(n, "|", m[1], "|", m[2]))
    end
    isempty(out) && error("no slots found in $path")
    fixed_sizes == 7 || error("expected fixed centers for the 7 odd sizes, found $fixed_sizes")
    length(out) == candidates + fixed_sizes || error("$candidates slot-shaped lines, only $(length(out) - fixed_sizes) parsed")
    # A label names one slot of a size: a duplicated line is an error, also when another
    # line was deleted and the count per size still fits.
    seen = Set{Tuple{Int,String}}()
    for l in out
        n_text, label, _ = split(l, "|"; limit=3)
        key = (parse(Int, n_text), String(label))
        key in seen && error("$(key[1])x$(key[1])x$(key[1]): slot $(key[2]) appears twice")
        push!(seen, key)
    end
    # The labels of a size are exactly the canonical ones (no other index, no leading zero)...
    for size in 2:15
        got = Set(label for (m, label) in seen if m == size)
        want = expected_labels(size)
        got == want || error("$(size)x$(size)x$(size): slot labels differ, missing $(sort!(collect(setdiff(want, got)))), unexpected $(sort!(collect(setdiff(got, want))))")
    end
    # ...and together the slots hold every facelet of the cube exactly once.
    for size in 2:15
        facelets = Int[]
        for l in out
            n_text, _, list = split(l, "|"; limit=3)
            parse(Int, n_text) == size && append!(facelets, parse.(Int, split(list, " ")))
        end
        sort!(facelets)
        facelets == collect(0:6size^2-1) || error("$(size)x$(size)x$(size): the slots do not hold each facelet 0..$(6size^2 - 1) exactly once")
    end
    # Every size must have exactly the slots its orbits have: a deleted line is an error.
    for size in 2:15
        got = count(l -> startswith(l, string(size, "|")), out)
        want = expected_lines(size)
        got == want || error("$(size)x$(size)x$(size): $got slot lines, expected $want")
    end
    sizes == collect(2:15) || error("expected the sizes 2..15, found $sizes")
    println("# Orbit64 V2 slot tables: n | slot | facelets, reference sticker first.")
    println("# Copied from FORMAT-V2-SLOTS.md of https://github.com/wstein/flix-orbit64")
    println("# (Apache-2.0, Werner Stein), which tools/format-v2/slots.jl there generates.")
    println("# Regenerate with tools/orbit64_v2_slots_fixture.jl; do not edit by hand.")
    foreach(println, out)
    println(stderr, "wrote $(length(out)) slots")
end

length(ARGS) == 1 || error("usage: orbit64_v2_slots_fixture.jl FORMAT-V2-SLOTS.md")
main(ARGS[1])
