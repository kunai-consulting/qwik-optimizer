import { $, component$ } from '@builder.io/qwik';
export const Header = component$(() => {
    return (
        <>
            <div onClick={(ctx) => console.log("1")}/>
            <div onClick={$((ctx) => console.log("2"))}/>
        </>
    );
});