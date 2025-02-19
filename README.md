# qwik-optimizer
Qwik Optimizer remake


**_DISCLAIMER_**: This is very much a work in progress so expect this space to change a lot.

## What
This is a ground up re-implementation of the original Qwik Optimzer.

## Why
The decision was made to mover from SWC to Oxide as the foundation for the Qwik Optimizer.  

In the process of doing this, we hope to make the internal Qwik Optimizer more modular and to be more idiomatic Rust.


## Modules

### prelude

This is a collection of common types and traits that are used throughout the Qwik Optimizer.

### component

This module currently contains metadata info for components.

## Usage

Not much save for the unit tests.

```shell 
cargo test
```