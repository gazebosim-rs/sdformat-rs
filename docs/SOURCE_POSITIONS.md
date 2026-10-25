# Source position cache

Valid semantic loading previously called xmlparser's full-prefix position scan
for every owned source record. Its API explicitly describes this operation as
expensive and intended for errors. Sampling a retained 1000-link/frame release
load stopped in that scan in four of five exploratory GDB samples. Generated
large-model timings also exposed superlinear growth. These observations motivate
a position cache; they do not establish full-loader or simulation performance.

Each private XML Document now retains a lexical prefix of TextPos records.
Requesting an element extends that prefix through its index, then returns the
cached position. The cursor's element starts remain ordered original byte offsets,
including the reviewed BOM correction. Each extension passes the next disjoint
source fragment to the same maintained xmlparser routine. A one-line fragment adds
its scalar column delta; a fragment containing LF supplies the final column and
adds its row delta. No Unicode/line-ending parser is replaced. Error positions
still use the original full-source fallback.

This adds storage proportional to the requested element prefix. It avoids eager
work for unqueried trailing opaque plugin children. The per-document RefCell has
no callbacks or nested position lookup while borrowed; its guard ends before the
caller receives TextPos or invokes a resolver. Documents/cursors remain private
and do not escape loading. Returned Sdf and owned Source values remain Send+Sync;
there is no global cache or shared mutable input state.

The initial eager prototype was retained with its measurements: it helped the
large model but slowed the opaque-tail workload. The lazy design and countercase
remain in the evidence. Exact source coordinates are checked against the old
full-source oracle across existing Unicode/BOM/EOL/import layouts, 64 additional
generated nested layouts and a 128-entry config-warning input. Existing source,
error, raw XML, semantic, noise and original-case assertions remain unchanged.
These are bounded generated regression inputs, not full fuzzing acceptance.

The benchmark runner creates fresh isolated build/target directories for both
source trees, uses the same release harness/toolchain and locked registry versions,
and pins only its own process/children. The default is the lowest available CPU. Input
reading, a validated warm load and a complete Sdf debug-state dump occur before
timing. The two full state files must match byte-for-byte. Every measured load
also checks counts, last-frame pose/attachment and checksum; document destruction
is timed. Seven alternating measured pairs follow a discarded pair by default.

```sh
python3 tools/benchmark_source_positions.py \
  --baseline /path/to/public-0fb9a140-source \
  --candidate "$PWD" --cpu 0
```

The workloads have 10, 100 or 1000 links and equally many chained frames, plus a
10-link/frame case with 10000 opaque trailing elements. Raw samples, extrema,
source/compiler/harness/lock/binary hashes and exact state hashes are retained.
Shared-host load and frequency are uncontrolled; small/opaque-case differences
must be assessed from the ranges and repeated runs, not described as universal
improvements. Debug equality is an experiment-specific regression check, not a
public format contract. A source-position optimization does not accept SDF-02,
P2, native full-loader equivalence or whole-engine performance. No dependency,
unsafe boundary, schema feature, runtime sensor or original-case retirement is
introduced. Independent source/benchmark review passed; strict staged/hosted
qualification remains required.

The recorded fresh-build run uses CPU 0: 1000 links/frames measured medians of
68.815 ms before versus 9.835 ms after (6.997 ratio). The opaque-tail median was
0.9% slower, while observed ranges overlap. These author samples and the earlier
eager regression are retained in reference/source_position_benchmark.json and
reference/source_position_eager_benchmark.json. They remain bounded shared-host
observations, not broad performance acceptance.

Independent review repeated all 179 Rust checks and all five native proofs, then
recreated fresh CPU-0 release targets. Its seven-pair 1000-link median was
51.053 ms before versus 9.778 ms after (5.22 ratio), with identical whole-state
hashes; opaque-tail was about 1% slower with overlapping ranges. Both measured
runs remain qualified by their shared host and bounded inputs.
