import { component$ } from '@qwik.dev/core';
import { server$ } from '@qwik.dev/router';
import { clientSupabase } from 'supabase';
import { Client } from 'openai';
import { secret } from './secret';
import { sideEffect } from './secret';

const supabase = clientSupabase();
const dfd = new Client(secret);

(function() {
	console.log('run');
	})();
	(() => {
	console.log('run');
	})();

sideEffect();

export const api = server$(() => {
	supabase.from('ffg').do(dfd);
});

export default component$(() => {
	return (
		<button onClick$={() => await api()}></button>
	)
	});
