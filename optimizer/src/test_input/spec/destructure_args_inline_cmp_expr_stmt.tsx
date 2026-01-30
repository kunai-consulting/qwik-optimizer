export default ({ data }: { data: any }) =>
    <div
      data-is-active={data.selectedOutputDetail === 'options'}
      onClick$={() => {
        data.selectedOutputDetail = 'options';
      }}
    />;
