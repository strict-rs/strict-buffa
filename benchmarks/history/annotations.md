# Release annotations

Why the numbers in [REPORT.md](REPORT.md) move. The data is now a **dense,
per-message-isolated matrix**: every message shape is measured against every
release (v0.1.0–v0.7.1), each built with only its own decoder compiled, at the
pinned toolchain (1.96.0) and `lto=true, codegen-units=1`, median of fifteen cores.
See [DESIGN.md](DESIGN.md) for the system and [README.md](README.md) for the
mechanics. Each release's harness lives on its `historical-benchmark/vX.Y.Z`
branch, so any cell is rebuildable.

Because each shape is isolated, the cross-release curves below are attributable to
buffa's own per-shape encode/decode code, not to which other messages happened to
share the benchmark binary. The charts shade a **±5% band** around each message's
baseline: that is the measured run-to-run noise floor on this hardware (median
core-to-core spread 2.6%, p90 6.6% across all 336 benchmarks), so a line that
stays inside the band never moved beyond noise. Movements that clear it are
discussed below.

## Headline cross-release findings (v0.1.0 → v0.7.1)

Improvements that clear the band:

- **PackedTile `decode_view` +43%** and **MediaFrame `json_encode` +40%** — the
  largest gains. JSON encoding improved broadly across shapes over the series, and
  the packed-tile view-decode path got substantially faster.
- ApiResponse `decode_view` +10%, AnalyticsEvent `merge` +10%, LogRecord
  `decode_view` +6% — eager-view decode and merge improved for several shapes.

Regressions that clear the band:

- **AnalyticsEvent `encode` −14%** and **`compute_size` −10%** — the deeply
  nested, repeated-submessage shape lost ground on the owned encode and size
  paths. This is the clearest real regression in the set and the one worth
  investigating.
- GoogleMessage1 `merge` −9%, AnalyticsEvent `json_encode` −7%, ApiResponse
  `compute_size` −6%.

Everything else is flat within the band: buffa's core binary `decode`/`encode`/
`merge` for the flat and string-heavy shapes has held steady across eight
releases, which is the reassuring headline.

## Why this replaced the earlier (sparse, coupled) history

An earlier version of this history built all shapes into one benchmark binary.
That made the per-shape numbers depend on which *other* shapes were present:
adding a message re-partitioned the compiler's inlining for the unchanged
decoders. It produced a convincing but false v0.7.1 regression — `MediaFrame`
`decode_view` read −13% purely because v0.7.1 added the `PackedTile` benchmark
message (proven by disassembly: removing PackedTile made MediaFrame's machine
code byte-identical to v0.7.0). Under per-message isolation that artifact is gone:
isolated `media_frame/decode_view` is flat across the whole series (≈44–48k MiB/s,
within spread). The dense isolated matrix exists so no cell can be contaminated
that way again, and so every shape has a full-history curve rather than starting
only at the release that added it to the suite.

## Caveats

These are medians of fifteen cores with per-benchmark spread recorded. The
headline movers above reproduced across two independent metal campaigns (an
earlier median-of-four run and this median-of-fifteen one), which is the main
evidence they are real rather than run artifacts; a delta inside the ±5% band is
noise. The matrix covers the seven portable operations (decode, merge, encode,
compute_size, decode_view, json_encode, json_decode) — the bespoke
`encode_view`/`build_encode` benchmarks use newer, view-encode APIs that did not
exist in older releases, so they are not part of the dense matrix and remain only
on the releases that natively support them.
