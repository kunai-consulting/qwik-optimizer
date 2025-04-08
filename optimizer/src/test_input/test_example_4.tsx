import { $, component$ } from '@builder.io/qwik';
export function App() {
    const Header = component$(() => {
        console.log("mount");
        return (
            <div onClick={$((ctx) => console.log(ctx))}/>
        );
    });
    return Header;
}
