import { $, component$ } from '@qwik.dev/core';
import {foo, bar as bbar} from "../state";
import * as dep2 from "dep2";
import dep3 from "dep3/something";

export const Header = component$(() => {
	return (
		<Header onClick={$((ev) => dep3(ev))}>
			{dep2.stuff()}{bbar()}
		</Header>
	);
});

export const App = component$(() => {
	return (
		<Header>{foo()}</Header>
	);
});
