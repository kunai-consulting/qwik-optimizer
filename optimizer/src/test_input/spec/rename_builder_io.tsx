import { $, component$ } from "@builder.io/qwik";
import { isDev } from "@builder.io/qwik/build";
import { stuff } from "@builder.io/qwik-city";
import { moreStuff } from "@builder.io/qwik-city/more/here";
import { qwikify$ } from "@builder.io/qwik-react";
import sdk from "@builder.io/sdk";

export const Foo = qwikify$(MyReactComponent);

export const Bar = $("a thing");

export const App = component$(() => {
	sdk.hello();
	if (isDev) {
		stuff()
	} else {
		moreStuff()
	}
	return "hi";
});
