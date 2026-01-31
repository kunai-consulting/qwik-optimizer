// don't transpile jsx with non-plain-object props
import { jsx } from '@qwik.dev/core';

export const App = () => {
    const props = {}
    return jsx('div', props, 'Hello Qwik');
}
