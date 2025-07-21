import { $, component$, type Component } from '@builder.io/qwik';

export const App = () => {
    const Header: Component = component$(() => {
        console.log("mount");
        return (
            <div onClick={$((ctx) => console.log(ctx))}/>
        );
    });
    return Header;
};
