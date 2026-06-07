# Introduction

`dioxus-swdir-tree` is a directory-tree explorer widget for Dioxus GUI apps,
ported from the design of `iced-swdir-tree`. It is a *viewer with gestures*:
it lazily lists directories one level per click, lets users select, navigate,
and drag entries — and leaves every actual file operation to your application.

This book is organized by audience: app developers who want a tree on screen,
integrators embedding the framework-free core elsewhere, and contributors
working on the crate itself.
