#!/usr/bin/env julia
# Writes tests/fixtures/orbit64_v2.txt from the reference vectors of flix-orbit64.
#
#     julia tools/orbit64_v2_fixture.jl PATH/TO/flix-orbit64/test/TestV2VectorData.flix \
#         > tests/fixtures/orbit64_v2.txt
#
# Needs only Julia's standard library. Each `(id, "state", "token")` row of the Flix
# file becomes one line `id state token`; the state text has no spaces.

function main(path)
    println("# Orbit64 V2 reference vectors: registry entry id, orbit coordinates, token.")
    println("# Copied from test/TestV2VectorData.flix of https://github.com/wstein/flix-orbit64")
    println("# (Apache-2.0, Werner Stein), which tools/format-v2/vectors.jl there generates.")
    println("# State text: key=v,v,...; with keys cp ct mp mf w1.. z1.. fr fm (see that file).")
    println("# Regenerate with tools/orbit64_v2_fixture.jl; do not edit by hand.")
    rows = 0
    for line in eachline(path)
        m = match(r"^\s*\((\d+), \"([^\"]*)\", \"([^\"]*)\"\),?\s*$", line)
        m === nothing && continue
        println(m[1], " ", m[2], " ", m[3])
        rows += 1
    end
    rows == 195 || error("expected 195 rows, found $rows")
end

length(ARGS) == 1 || error("usage: orbit64_v2_fixture.jl TestV2VectorData.flix")
main(ARGS[1])
