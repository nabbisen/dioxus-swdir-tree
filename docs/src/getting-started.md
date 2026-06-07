# Getting started

> v0.1 ships the core state machine only. The `DirectoryTreeView` component
> arrives in v0.3.0; this chapter will then gain a copy-paste Dioxus example.

Add the flagship crate:

```toml
[dependencies]
dioxus-swdir-tree = "0.1"
```

Create a tree and expand its root synchronously (fine for scripts and tests):

```rust
use dioxus_swdir_tree::DirectoryTree;

let mut tree = DirectoryTree::new("/home/me/projects");
tree.expand_blocking(std::path::Path::new("/home/me/projects"));
for (node, depth) in tree.visible_rows() {
    println!("{}{}", "  ".repeat(depth as usize), node.file_name().display());
}
```
