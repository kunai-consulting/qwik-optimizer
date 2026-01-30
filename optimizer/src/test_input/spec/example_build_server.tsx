import { component$, useStore, isDev, isServer as isServer2 } from '@qwik.dev/core';
import { isServer, isBrowser as isb } from '@qwik.dev/core/build';
import { mongodb } from 'mondodb';
import { threejs } from 'threejs';

import L from 'leaflet';

export const functionThatNeedsWindow = () => {
  if (isb) {
    console.log('l', L);
    console.log('hey');
    window.alert('hey');
  }
};

export const App = component$(() => {
	useMount$(() => {
		if (isServer) {
			console.log('server', mongodb());
		}
		if (isb) {
			console.log('browser', new threejs());
		}
	});
	return (
		<Cmp>
			{isServer2 && <p>server</p>}
			{isb && <p>server</p>}
		</Cmp>
	);
});
