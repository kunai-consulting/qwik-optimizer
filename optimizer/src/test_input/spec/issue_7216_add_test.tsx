import { component$ } from '@builder.io/qwik';
export default component$((props) => {
  return (<p 
		onHi$={() => 'hi'} 
		{...props.foo} 
		onHello$={props.helloHandler$} 
		{...props.rest} 
		onVar$={props.onVarHandler$} 
		onConst$={() => 'const'} 
		asd={"1"}
	/>);
});
