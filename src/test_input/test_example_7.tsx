import { $, component$ } from '@builder.io/qwik';

export const Header = component$(() => {
    console.log("mount");
    return (
        <div onClick={$((ctx) => console.log(ctx))}/>
    );
});

 const App = component$(() => {
    return (
        <Header/>
    );
});

