# Display filters and the cache

Three modes: `FoldersOnly`, `FilesAndFolders` (default),
`AllIncludingHidden`. Hiddenness is decided at scan time — dotfiles on Unix;
`FILE_ATTRIBUTE_HIDDEN` or dotfile on Windows — and stored on each
`LoadedEntry`.

The tree caches the **raw, unfiltered** entry list per scanned directory.
`set_filter` therefore rebuilds the visible tree from the cache with zero
I/O, preserving expansion state for every node that remains visible.
