# Authoring benchmarks

Measured runs of AI agents authoring library icons with the `generative-icons` CLI and the hosted engine. The raw data for each run sits beside this page.

## Category fill, 5 October 2026

Every library category with fewer than 20 icons was brought up to 20 with new spec v2 icons: 136 icons across 14 categories. The library grew from 1,005 to 1,141 icons. Data: [benchmark-category-fill-2026-10.json](benchmark-category-fill-2026-10.json).

**How it ran:**
- **Agents:** one AI agent per category, 14 in all, at most 8 at a time.
- **Engine:** four engine slots, each holding one engine call at a time, with two agents sharing each slot.
- **Brief:** every agent followed the same brief:
  1. Choose concepts after searching the whole library.
  2. Claim the ids in a shared list.
  3. Draw in the set's style.
  4. Animate an event.
  5. Run `check`, `draft` and `build`, then the library's visual check for the set (`verify-visual` rules).
- **Logging:** a wrapper logged every CLI command (2,358 in all), with its wall time, its wait for an engine slot, its outcome and the engine's own timing.
- **Library:** each finished category was added with `library adopt NN --add` and passed `library verify` and `verify-visual`.

| Measure | Result |
| --- | --- |
| Icons delivered | 136 of 136, in 14 categories; every category now has at least 20 |
| Wall time | 68.6 min from the first launch to the last category (473.7 agent-minutes) |
| Throughput | 2.0 icons per minute overall; 0.29 icons per agent-minute |
| Engine calls per icon | 1.54 drafts and 1.41 builds |
| First build passes | 97% |
| First visual check passes | 99% (the set's own rules at every size, style, weight and theme) |
| Median engine time | 8.4 s per build (round trip 5.2 s: 1.2 s before the script, 0.09 s in it, 3.8 s after); 4.0 s per draft |
| Engine time in total | 50 min across all calls, spread over 4 slots: the agents' thinking, not the engine, set the pace |
| Waiting for a slot | 6.3 min in total over 401 engine calls |
| Library searches | 954, to rule out duplicates; agents dropped 35 candidate concepts that an existing icon already covered or that had no clear event |

| Category | Set | Added | Wall (min) | Drafts per icon | Builds per icon | Dropped concepts |
| --- | --- | --- | --- | --- | --- | --- |
| Get help | 23 | 15 | 47.5 | 1.67 | 1.80 | 2 |
| Essentials | 03 | 12 | 46.0 | 1.42 | 1.67 | 0 |
| Scheduling & time tools | 34 | 12 | 33.9 | 1.42 | 1.42 | 3 |
| Communication & social | 33 | 11 | 34.5 | 1.45 | 1.36 | 0 |
| Actions | 04 | 10 | 31.5 | 1.60 | 1.30 | 1 |
| Voice | 16 | 10 | 41.3 | 2.00 | 1.80 | 7 |
| Files & transfers | 24 | 10 | 39.2 | 1.90 | 1.60 | 5 |
| Infrastructure & devices | 32 | 10 | 29.8 | 1.30 | 1.00 | 5 |
| Accessibility & reading | 30 | 10 | 32.6 | 1.40 | 1.60 | 5 |
| Forms & input | 27 | 9 | 27.7 | 1.56 | 1.00 | 1 |
| Account & privacy | 26 | 8 | 31.9 | 1.25 | 1.00 | 1 |
| Annotation arrows | 21 | 8 | 34.1 | 1.62 | 1.00 | 4 |
| Developer & version control | 31 | 7 | 28.1 | 1.43 | 1.57 | 1 |
| Tables & analytics | 28 | 4 | 15.5 | 1.25 | 1.00 | 0 |

**Against the earlier run.** The HotelOS dogfood (2 October 2026) had four agents write v1 JS batches, with no library-level verification. It made 12 icons in 19 minutes with 24 builds:

| Measure | HotelOS (v1) | Category fill (spec v2) |
| --- | --- | --- |
| Icons per agent-minute | 0.16 | 0.29 |
| Builds per icon | 2.0 | 1.41 |

Category fill also cleared the stricter bar: every icon passed its set's visual rules and the library's checks.

**What failed, and what it points to.** 13 of 192 builds failed:
- **6 were the engine or network:** five `ENGINE_UNAVAILABLE` and one network error. A retry passed each time.
- **4 were icons drawn only with fills (app-grid, braille).** They have no outline stroke, so the paint contract rejects them at build time. `check` should warn about this earlier.
- **The rest:** one clipped icon, and two without a recorded error.

`check` errors seen while drafting:
- shape mistakes: `V2_SCHEMA`, `MORPH_TOPOLOGY`, `UNKNOWN_NAME`
- the knockout limits: `KNOCKOUT_RIGS`, `KNOCKOUT_TRACKS`
- `REST_VALUE` and `MOVES_AFTER_REST`
- `COMBINED_PATH_TRACKS`

All of them were caught offline before any engine call.
