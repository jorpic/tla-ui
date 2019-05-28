
Using codemirror prevents us from prerendering. Building works only with
`--no-prerender` flag.

Consider configuring code splitting to fix this.


Building
--------

```
cd tla-parser ; wasm-pack build --target web ; cd -
cd ui ; preact build -no-prerender - ; cd -
```

TODO
----
  - Save to IDB
  - Add tla-parser
  - side pane with parse results

GOALS
  - code colouring
  - unicode ligatures
  - model-config editor
  - parse model checking results
  - visualize model checking trace
  - tla+ parser with human readable errors
    - could be integrated with other editors: vim, emacs, source.
  - online parsing / intellisense
     - undefined identifier
     - type error
  - code folding
