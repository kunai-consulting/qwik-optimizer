/* @jsxImportSource react */

import { qwikify$ } from './qwikfy';

export const App = () => (
	<div onClick$={()=>console.log('App')}></div>
);

export const App2 = qwikify$(() => (
	<div onClick$={()=>console.log('App2')}></div>
));
