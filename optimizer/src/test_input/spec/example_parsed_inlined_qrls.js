import { componentQrl, inlinedQrl, useStore, jsxs, jsx, useLexicalScope } from '@qwik.dev/core';

export const App = /*#__PURE__*/ componentQrl(inlinedQrl(()=>{
	useStyles$(inlinedQrl(STYLES, "STYLES_odz7dfdfdM"));
	useStyles$(inlinedQrl(STYLES, "STYLES_odzdfdfdM"));

	const store = useStore({
		count: 0
	});
	return /*#__PURE__*/ jsxs("div", {
		children: [
			/*#__PURE__*/ jsxs("p", {
				children: [
					"Count: ",
					store.count
				]
			}),
			/*#__PURE__*/ jsx("p", {
				children: /*#__PURE__*/ jsx("button", {
					onClick$: inlinedQrl(()=>{
						const [store] = useLexicalScope();
						return store.count++;
					}, "App_component_div_p_button_onClick_odz7eidI4GM", [
						store
					]),
					children: "Click"
				})
			})
		]
	});
}, "App_component_Fh88JClhbC0"));

export const STYLES = ".red { color: red; }";
