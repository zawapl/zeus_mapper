# `pkware` module notes

Implementation notes for anyone revisiting this module - where it comes from, how it was
validated, and the one known gap in its output fidelity. The rustdoc comments in this module are
kept deliberately high-level (what the code does, how to use it); this file is where the
lower-level reverse-engineering detail lives instead.

## Where this comes from

This module reimplements the PKWARE Data Compression Library ("implode"/"explode") algorithm used
by the Impressions Games engine (Caesar 3, Zeus, Pharaoh all share it) for compressed sections of
`.pak`/`.set`/`.map` files. It's wired into this crate's read/write paths in place of a
general-purpose PKWARE crate dependency, because the real game rejects files whose unmodified
compressed data doesn't match its own original bytes exactly - a validly-decodable re-encoding of
the same content isn't enough, since compression isn't unique and a different (but equally valid)
sequence of match/literal choices produces different bytes. So this module needs to match the real
game's own compressor's specific choices, not just the DCL format in the abstract.

The implementation was derived directly from decompiling `Zeus.exe`'s own compressor and
decompressor (via Ghidra), field-by-field and function-by-function, rather than from any other
project's reimplementation of the algorithm.

## Function map (`Zeus.exe` virtual addresses, as of the version of the game this was checked
against)

Compress side:

| Address    | Role                                                                         |
|------------|------------------------------------------------------------------------------|
| `0x5c6300` | Top-level compress entry point (sets up buffers, calls open, reports errors) |
| `0x5c96c0` | Open/init - builds the codeword tables, copies in the offset tables          |
| `0x5c9860` | Main match-search-and-emit loop (`implode.rs`'s `run`)                       |
| `0x5ca0a0` | Input analysis / hash-bucket sort (`analyze_input`)                          |
| `0x5c9be0` | Match search, incl. the KMP-assisted second phase (`determine_copy`)         |
| `0x5c9f20` | Bit writer, LSB-first byte-at-a-time flush (`bit_io::BitWriter`)             |
| `0x5c9fb0`, `0x5ca7e0` | `memset`/`memcpy` equivalents, not algorithm-specific              |

Decompress side:

| Address    | Role                                                                         |
|------------|--------------------------------------------------------------------------|
| `0x5c63d0` | Top-level decompress entry point, mirrors `0x5c6300`                        |
| `0x5ca160` | Open/init - reads the header, builds the length/offset jump tables          |
| `0x5ca2e0` | Main token loop (`explode.rs`'s `decode`)                                   |
| `0x5ca3e0` | Token decode: literal vs. copy-length dispatch (`decode_next_token`)        |
| `0x5ca590` | Copy-offset decode (`get_copy_offset`)                                      |
| `0x5ca610` | Bit reader/consumer, 16-bit sliding window refill (`bit_io::BitReader`)     |

`0x5ca160` also builds a set of ASCII-literal jump tables via a call to a helper at `0x5ca6e0`,
but only when the header's `has_literal_encoding` flag is set - Zeus's own encoder/decoder pair
never sets it, so that whole path is dead code for real save files and has no equivalent in this
crate (see the `has_literal_encoding == 0` assumption noted in `implode.rs`/`explode.rs`).

The two PKWARE constant tables (`tables.rs`'s `COPY_OFFSET_BITS`/`COPY_OFFSET_CODE`/
`COPY_LENGTH_BASE_BITS`/`COPY_LENGTH_BASE_CODE`/`COPY_LENGTH_BASE_VALUE`/`COPY_LENGTH_EXTRA_BITS`)
live in `Zeus.exe`'s `.data` section as two near-identical clusters at VAs `0x662478`/`0x6628f0`,
1144 bytes apart - one copy read by the compressor's open routine, one by the decompressor's.

Every one of the above was traced and cross-checked field-by-field against this module's Rust
logic - see "Known gap" below for the one place they still diverge.

## How to reproduce the decompilation

Ghidra and a JDK are both portable (unzip and run, no system install, no root needed):

1. Download a portable JDK (check the target Ghidra release's minimum-version requirement first)
   and the latest Ghidra release ZIP from `https://github.com/NationalSecurityAgency/ghidra/releases`.
2. Extract both. Set `JAVA_HOME` to the extracted JDK.
3. Run headless analysis: `<ghidra_dir>/support/analyzeHeadless <project_dir> <project_name>
   -import "$ZEUS_HOME/Zeus.exe" -analyze`. Expect real time (10-30+ minutes) for a ~2.5MB binary -
   don't kill it early.
4. For querying after import, use headless post-analysis scripting: `-process <project_name>
   -postScript YourScript.java -scriptPath <dir>` against the already-imported project (no need to
   re-`-import`/re-analyze). Ghidra's headless scripting API is Java (Jython/PyGhidra may not be
   available depending on the build). A short script using `DecompInterface` + `FunctionManager`
   can decompile arbitrary functions by address and list call/callee cross-references - useful both
   for re-deriving a function from scratch and for sanity-checking this module's Rust code against
   the real one after a change. If a GUI is available instead, the interactive CodeBrowser is much
   faster for this kind of exploratory work.

If the analyzed project directory is ever on a read-only mount, headless scripting against it
directly fails with `ReadOnlyException` - copy the project directory to a writable location first
(the Ghidra/JDK installs themselves can stay wherever they are and be used read-only).

## Known gap: not byte-identical on every real field

This module's own tests are all pure unit tests (no `ZEUS_HOME`/real files needed) - validating
against real game files is handled by a whole-file round-trip regression test in
`file_data::pak_data`'s tests instead, so there's one place for that rather than two. A
now-removed dedicated per-field regression test that used to live here measured 2381/2474 (~96%)
of real compressed fields recompressing byte-identically to the original file, which is where the
figures below come from; that measurement is historical rather than something re-checked on every
run, but the underlying cause is understood well enough that it doesn't need re-measuring to trust.

The mismatches all share one exact, understood cause: the very last
copy-match in a stream, whenever it's long enough to hit the format's maximum match length,
resolves a same-length tie between two candidate back-reference offsets differently than the real
file does. Both encodings decode to identical content - this is the general "compression isn't
unique" phenomenon, not a bug - but the real compressor's tie-break reads a few bytes past the end
of its own input, out of its stack frame's working buffer for tile data, which is genuinely
uninitialized memory there (confirmed via decompilation: the only zeroing call in the real
compressor's main loop targets a small, different, fixed region that ends exactly where that
working buffer begins, not the buffer itself or its trailing overrun margin). That overrun content
isn't recoverable from the file format, so this port doesn't attempt to replicate it - it
zero-fills its equivalent buffer instead, for deterministic, safe behavior in Rust (see
`INPUT_DATA_SIZE`'s doc comment in `implode.rs`). For most fields this never matters (the tie only
breaks differently when a match happens to run all the way into that overrun region), which is why
the large majority still match.

One thing that was tried and ruled out: chaining two real fields' compression through one shared,
never-re-zeroed scratch buffer (simulating "the real game reuses one dirty buffer call to call")
did not reproduce the real offset choice for the case tested - for any field spanning more than
one internal compression chunk, the bytes just past its own logical end are actually a copy of
that same field's own earlier chunk tail (via this port's already-correct sliding-window logic),
not another field's data. The truly foreign, unrecoverable overrun is specifically whatever sits
immediately adjacent to the real input buffer in the compiled struct's memory layout - closing
this gap further would require reconstructing `Zeus.exe`'s precise stack frame layout and
restructuring this port to use one matching flat buffer instead of separate allocations, so an
out-of-bounds read naturally lands on the equivalent bytes. Not attempted; likely diminishing
returns, and possibly partially irreducible (the very first compression call in a session would
still depend on whatever unrelated code touched that stack depth before any compression ever ran,
which isn't derivable from the file format at all).

This is a known, understood, and currently-accepted gap, not a regression to chase without new
evidence. Only lower an asserted floor in these tests if a real regression is found, and only
raise it if a fix for this specific gap is landed.
