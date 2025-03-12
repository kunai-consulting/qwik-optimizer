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


## Questions on Previous Implementation

### test_example_8

#### Input

```typescript jsx
import { $, component$ } from '@builder.io/qwik';

export const Header = component$(() => {
    return $((hola) => {
        const hola = this;
        const {something, styff} = hola;
        const hello = hola.nothere.stuff[global];
        return (
            <Header/>
        );
    });
});
```

####  test.tsx_Header_component_1_2B8d0oH9ZWc.tsx 

Why is the `const hello = hola.nothere.stuff[global];` assignment changed to a call, `hola.nothere.stuff[global];`? 

```typescript
import { Header } from "./test";
export const Header_component_1_2B8d0oH9ZWc = (hola)=>{
    const hola = this;
    const { something, styff } = hola;
    hola.nothere.stuff[global];
    return <Header/>;
};
export { _hW } from "@builder.io/qwik";
>>>>>>> 62603bb (WIP: Testing)
```