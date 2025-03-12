import { $, component, onRender } from '@builder.io/qwik';

export const renderHeader = $(() => {
    return (
        <div onClick={$((ctx) => console.log(ctx))}/>
    );
});
const renderHeader = component($(() => {
    console.log("mount");
    return render;
}));

