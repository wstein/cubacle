#!/usr/bin/env julia
# Writes tests/fixtures/orbit64_v2_picture_facelets.txt from the independent picture vectors of
# flix-orbit64.
#
#     julia tools/orbit64_v2_picture_fixture.jl \
#         PATH/TO/flix-orbit64/test/TestV2FaceletPictureData.flix \
#         > tests/fixtures/orbit64_v2_picture_facelets.txt
#
# Needs only Julia's standard library. Each
# `(id, "moves", "facelets", "token", "canonical"),` row of the Flix file, which spans three
# lines there, becomes one line `id|moves|facelets|token|canonical`; the move text and the
# canonical facelets may be empty. The output is the same bytes on every run.

# The rows of the reference file: 13 sizes (3..15), a canonical and a framed entry each, with
# several move sequences per size. A reference file with another count is a different file.
const EXPECTED_ROWS = 174

function main(path)
    # Collect and check every row first, then print: a failed run writes
    # nothing, so redirecting to the fixture never leaves a truncated file.
    text = read(path, String)
    row = r"^\s*\((\d+), \"([^\"|]*)\", \"([^\"|]*)\",\s*\"([^\"|]*)\",\s*\"([^\"|]*)\"\),?[ \t]*$"m
    out = [join((m[1], m[2], m[3], m[4], m[5]), "|") for m in eachmatch(row, text)]
    # Lines that look like the start of a row, whether or not the row parses.
    candidates = count(line -> occursin(r"^\s*\(\d+,", line), split(text, '\n'))
    rows = length(out)
    rows > 0 || error("no rows found in $path")
    rows == candidates || error("$candidates row-shaped lines, only $rows parsed")
    rows == EXPECTED_ROWS || error("$rows rows, expected $EXPECTED_ROWS")
    length(unique(out)) == rows || error("$(rows - length(unique(out))) duplicated rows")
    # Every chunk of rows is defined once and used once by the aggregate that lists it.
    for name in unique(m.match for m in eachmatch(r"rows\d+p\d+", text))
        count(Regex("\\b$name\\b"), text) == 2 ||
error("$name is not defined and used exactly once")
    end
    println("# Orbit64 V2 picture facelet vectors: registry entry id | moves | facelets | token |")
    println("# canonical facelets. Copied from test/TestV2FaceletPictureData.flix of")
    println("# https://github.com/wstein/flix-orbit64 (Apache-2.0, Werner Stein), which")
    println("# tools/format-v2/facelet_picture_vectors.jl there generates.")
    println("# The canonical column is empty when the facelets are already canonical.")
    println("# Regenerate with tools/orbit64_v2_picture_fixture.jl; do not edit by hand.")
    foreach(println, out)
    println(stderr, "wrote $rows rows")
end

length(ARGS) == 1 || error("usage: orbit64_v2_picture_fixture.jl TestV2FaceletPictureData.flix")
main(ARGS[1])
