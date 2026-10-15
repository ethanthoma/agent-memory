# Fictional retrieval diagnostic

This fixture uses invented library policies. It contains 10 notes and 12 task
queries: four direct lookups, four paraphrases, two related situations, and two
questions with no matching fact. It is a smoke test for retrieval behavior,
not a benchmark of real project memory or agent answer quality.

Each method returns at most three notes and 2048 rendered UTF-8 bytes. Plain
notes contain a fact and source label; cued notes also contain a recall cue.
The substring method requires a literal case-insensitive match. BM25 ranks
token matches and may return a candidate even for an unsupported question.

| Method | Direct hit | Indirect hit | Associative hit | MRR@3 | Unknown abstention | Mean context bytes |
| --- | --- | --- | --- | --- | --- | --- |
| substring-plain | 4/4 | 0/4 | 0/2 | 0.400 | 2/2 | 28 |
| BM25-plain | 4/4 | 2/4 | 2/2 | 0.800 | 0/2 | 113 |
| substring-cues | 4/4 | 0/4 | 0/2 | 0.400 | 2/2 | 47 |
| BM25-cues | 4/4 | 4/4 | 2/2 | 1.000 | 0/2 | 227 |

Run `agent-memory-eval evaluation` to reproduce these numbers. The fixture is
small and authored alongside its queries, so the values describe only this
example. They do not justify a general accuracy claim. The unsupported
questions demonstrate why ranked results must be treated as candidates.
