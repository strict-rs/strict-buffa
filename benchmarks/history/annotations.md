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
discussed below. Spread is uneven across operations — `compute_size` is the
tightest (p90 2.7%), the binary-`encode` and JSON paths the noisiest (p90 ~9–10%)
— so the per-operation summary in [REPORT.md](REPORT.md)'s "Measurement spread"
table, and the per-benchmark spread in `runs/*.json`, are where to check how far
a given number can be trusted, rather than reading it off the chart.

## Headline cross-release findings (v0.1.0 → v0.7.1)

A movement counts as real only if it is large *for its operation*, sits on a
layout-stable operation (or is corroborated by one), and **persists** across
releases. As "Code layout is the dominant noise source" below shows, clearing the
±5% band is not sufficient — build layout alone can move a number ~20%. By the
stricter test, two findings stand:

- **AnalyticsEvent `encode` −14% / `compute_size` −10%** — a real regression. It
  is a step down at v0.4.0 that holds through v0.7.1, and `compute_size` is a
  layout-stable operation (≈3% spread), so its 10% drop clears the noise by 3–4×
  and corroborates the noisier `encode` figure. The deeply nested,
  repeated-submessage shape genuinely lost ground on the owned encode/size paths —
  the one result worth investigating.
- **PackedTile `decode_view` +43% at v0.7.1** — flat (~170 MiB/s) from v0.1.0
  through v0.7.0, then a single-release jump to ~246 at v0.7.1, consistent with the
  packed-varint reserve work in that release. `decode_view` is layout-stable, so a
  43% step is well clear of noise; but it is the latest release, so "persists"
  isn't confirmable yet.

Everything else is within build-layout noise: the +5–10% `decode_view` / `merge`
wiggles, the `encode`-only movements, and *all* of `json_encode` / `json_decode`
(demoted — see below) are not distinguishable from how the compiler happened to
place the code. buffa's core binary `decode` / `merge` for the flat and
string-heavy shapes is flat across eight releases — the reassuring headline.

## Code layout is the dominant noise source

Per-message isolation removed cross-message coupling, but a subtler effect is now
the limiting factor on resolution: the **placement** of otherwise-identical code.
The clearest case is `json_encode` for the string/scalar-heavy shapes, which
*flaps* — LogRecord and ApiResponse both run ~24% faster at v0.5.0 and v0.7.0 and
slower at the releases between, in lockstep, which no real code change would do.

Disassembling the fast (v0.5.0) and slow (v0.6.0) isolated LogRecord binaries
settled it: **2390 of 2393 functions are byte-identical** after normalising
addresses. The only three that differ are `__rust_alloc_zeroed`, a `raw_vec`
error path, and `main` — none in the encode path. `serialize_str` and
`format_escaped_str`, the string-escaping hot loop that dominates this shape, are
identical instruction-for-instruction, just located at different addresses. A
trivial source change between releases shifts a few glue functions' sizes, which
shifts every later function's address, so the *same* hot loop lands at a different
alignment / cache-line packing and runs up to 24% faster or slower **with no code
change**.

Two follow-up experiments pinned this down:

- **It's the build, not the measurement.** Re-measuring the same binaries
  one-at-a-time on an idle box (no parallel waves) reproduced the gap exactly
  (waved 42.6 / 54.6 µs vs 1-up 42.3 / 54.7 µs). Concurrency, co-scheduling, and
  sample count change nothing — the number is a fixed property of the binary.
- **A rebuild of the identical source can flip it.** Building the very same commit
  in a different directory produced binaries with *no* gap (≈37.5 µs for both
  versions). Same source, flags, and toolchain — a different incidental layout, a
  different result. So the gap isn't even tied to the source version; it is the
  luck of one build's function placement.

What this means for reading the history:

- **The trust threshold is ±20%, not ±5%, for the layout-sensitive operations.**
  Build layout alone moves `json_encode` ~24%, so on those paths only a change
  that clears ~20% *and persists* across releases is real; everything inside that
  band is compiler noise. (The ±5% chart band is the *typical* run-to-run floor
  for the stable operations, not the bound for the sensitive ones.)
- **Weight by operation.** `compute_size`, `decode`, `merge`, and `decode_view`
  are layout-stable and trustworthy; `encode` is borderline; `json_encode` /
  `json_decode` are layout-dominated and are **demoted** — no charts, kept only in
  REPORT.md's "directional only" section. The "Measurement spread" table ranks all
  of them.
- **Why JSON is the worst — measured, not guessed.** `perf stat` on a slow vs a
  64-byte-aligned build of the *same* code (IPC 2.87 vs 3.55) traces the penalty to
  the **µop cache (DSB)**, not the instruction cache. The Topdown breakdown puts the
  slow layout's stall in Fetch *Bandwidth* (21.7% of slots vs 11.5%), with ~2× the
  DSB→legacy-decoder (MITE) switch penalty (`dsb2mite_switches.penalty_cycles`),
  while Fetch *Latency* and the i-cache / cache-line miss counters barely move. The
  serde serialize path is a dense tree of many small functions (string escaping,
  int/float formatting), so its hot loop is unusually sensitive to how the µop cache
  packs it: a placement shift tips it out of the DSB into the slower legacy decoder.
  (An earlier "cache-line" reading was wrong — the counters say front-end
  *bandwidth*, not *latency*.)

Two things *do* converge the fast layout: 64-byte block alignment
(`-align-all-nofallthru-blocks=6`) and a BOLT pass — both restore clean DSB delivery
and pull the slow ~51 µs builds back to ~37 µs (BOLT even with a no-LBR profile). We
apply neither, because each measures a *best-achievable* layout rather than what a
plain `cargo build` ships, and the stable operations don't need it. So treat this
history as a reliable detector of **large, persistent** shifts on the stable
operations, and nothing finer.

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

These are medians of fifteen cores with per-benchmark spread recorded.
Reproduction across the two metal campaigns (median-of-four and median-of-fifteen)
rules out random run noise — but **not** layout artifacts, which are deterministic
per binary and reproduce just as cleanly (the `json_encode` flap above did). The
real-versus-layout test is persistence across *releases*, not reproduction across
*runs*. The matrix covers the seven portable operations (decode, merge, encode,
compute_size, decode_view, json_encode, json_decode) — the bespoke
`encode_view`/`build_encode` benchmarks use newer, view-encode APIs that did not
exist in older releases, so they are not part of the dense matrix and remain only
on the releases that natively support them.
