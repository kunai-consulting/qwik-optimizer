# qwik-optimizer
Qwik Optimizer remake


## Component Hash Generation

Current hashing algo is just Rust's `DefaultHasher`.

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
```