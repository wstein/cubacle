#!/usr/bin/env julia
# Writes tests/fixtures/orbit64_v2_slots.txt from the normative slot tables of flix-orbit64.
#
#     julia tools/orbit64_v2_slots_fixture.jl PATH/TO/flix-orbit64/FORMAT-V2-SLOTS.md \
#         > tests/fixtures/orbit64_v2_slots.txt
#
# Needs only Julia's standard library. Each slot line of the markdown tables becomes one
# line `n|label|facelets` (for example `4|W1 0|1 82`, `5|Z[1,2]|7 11 ...`).

function main(path)
    # Collect and check everything first, then print (see orbit64_v2_fixture.jl).
    out = String[]
    sizes = Int[]
    n = 0
    in_block = false
    candidates = 0
    for line in eachline(path)
        m = match(r"^## (\d+)x\1x\1$", line)
        if m !== nothing
            n = parse(Int, m[1])
            push!(sizes, n)
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
    length(out) == candidates || error("$candidates slot-shaped lines, only $(length(out)) parsed")
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
