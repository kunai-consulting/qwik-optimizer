import { $, component$ } from '@builder.io/qwik';
export const App = () => {
    const Header = component$(() => {
        console.log("mount");
        return (
            <div onClick={$((ctx) => console.log(ctx))}/>
        );
    });
    return Header;
};

