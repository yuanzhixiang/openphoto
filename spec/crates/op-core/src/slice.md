# slice.rs
User slices carry stable per-document IDs, names, bounds and URL/target/alt text. create clips bounds to the canvas and rejects empty regions. rgba8 returns dimensions and tightly packed composite pixels for a slice, without file I/O. Slices participate in Document snapshots. Auto slices, canvas-transform remapping and file export UI are not implemented.

PSD persistence uses version-6 resource 1050; later descriptor versions remain unsupported.

from_guides replaces user slices with the grid formed by finite in-canvas ruler guides and canvas edges, rounding and deduplicating coordinates. Repeating an identical grid is a no-op. IDs fit PSD u32 fields.
