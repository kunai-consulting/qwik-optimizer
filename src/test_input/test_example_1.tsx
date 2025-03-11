// @ts-ignore
import {$, component, onRender} from '@builder.io/qwik';

// @ts-ignore
export const renderHeader = $(() => {
    return (
        <div onClick={$((ctx) => console.log(ctx))}/>
    );
});
// @ts-ignore
const renderHeader = component($(() => {
    console.log("mount");
    return render;
}));
