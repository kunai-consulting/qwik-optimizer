import { $, component$, useStore, useSignal } from '@qwik.dev/core';
export default component$((props: { id: number }) => {
      const renders = useStore(
        {
          count: 0,
        },
        { reactive: false }
      );
      renders.count++;
      const rerenders = renders.count + 0;
      const Id = (props: any) => <div>Id: {props.id}</div>;
      return (
        <>
          <Id id={props.id} />
          {rerenders}
        </>
      );
    });
