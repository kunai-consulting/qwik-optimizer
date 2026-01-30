import { component$, $ } from '@qwik.dev/core';
import json from "./foo.json" assert { type: "json" };

export const Greeter = component$(() => {
	return json;
});
