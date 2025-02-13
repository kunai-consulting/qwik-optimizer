# Requirements

Also see https://github.com/QwikDev/qwik-evolution/discussions/214

- standalone package with separate platform binary packages for linux, mac, and windows, like sharp and esbuild
  - qwik will import it as a dependency with pinned versions

# What the optimizer does

- parse js/ts/jsx/tsx code
- split code into segments which are or are not kept in the same file
- extract scope
- remove named segments like `server$` depending on server/client
- transform code in each segment, including replacing build constants with actual values
- separate `sync$` segments
- return a list of transformed segments
- rename builder.io imports to qwik.dev imports
- come up with hashed names for each segment
  - this includes the "path" inside the file, like component_useTask_1
- handle dev/prod: add dev metadata
- handle library mode
- actually optimize:
  - re-structure props destructuring
  - forward signal props
- add PURE annotations
- convert `jsx()` calls to `_jsxSplit` and `_jsxSorted`
- forward signal props via `_fnSignal` and `_wrapProp`

## More details

- accept a code string with config:
  - entry strategy: explains where to put segments and which ones go together
