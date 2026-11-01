#!/usr/bin/env julia
# Writes tests/fixtures/orbit64_v2_facelets.txt from the independent facelet vectors of flix-orbit64.
#
#     julia tools/orbit64_v2_facelet_fixture.jl PATH/TO/flix-orbit64/test/TestV2FaceletData.flix \
#         > tests/fixtures/orbit64_v2_facelets.txt
#
# Needs only Julia's standard library. Each `(id, "moves", "facelets", "token", "canonical")`
# row of the Flix file becomes one line `id|moves|facelets|token|canonical`; the move text
# and the canonical facelets may be empty.

function main(path)
    # Collect and check every row first, then print: a failed run writes
    # nothing, so redirecting to the fixture never leaves a truncated file.
    out = String[]
    candidates = 0 # lines that look like a row, whether or not they parse
    for line in eachline(path)
        occursin(r"^\s*\(\d+,", line) && (candidates += 1)
        m = match(r"^\s*\((\d+), \"([^\"|]*)\", \"([^\"|]*)\", \"([^\"|]*)\", \"([^\"|]*)\"\),?\s*$", line)
        m === nothing && continue
        push!(out, join((m[1], m[2], m[3], m[4], m[5]), "|"))
    end
    # The row count comes from the input: every row-shaped line must parse.
    rows = length(out)
    rows > 0 || error("no rows found in $path")
    rows == candidates || error("$candidates row-shaped lines, only $rows parsed")
    println("# Orbit64 V2 facelet vectors: registry entry id | moves | facelets | token | canonical facelets.")
    println("# Copied from test/TestV2FaceletData.flix of https://github.com/wstein/flix-orbit64")
    println("# (Apache-2.0, Werner Stein), which tools/format-v2/facelet_vectors.jl there generates.")
    println("# The canonical column is empty when the facelets are already canonical.")
    println("# Regenerate with tools/orbit64_v2_facelet_fixture.jl; do not edit by hand.")
    foreach(println, out)
    println(stderr, "wrote $rows rows")
end

length(ARGS) == 1 || error("usage: orbit64_v2_facelet_fixture.jl TestV2FaceletData.flix")
main(ARGS[1])
