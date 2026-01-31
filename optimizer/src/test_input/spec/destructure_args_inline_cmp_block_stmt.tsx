export default ({ data }: { data: any }) => {
  return (
    <div
      data-is-active={data.selectedOutputDetail === 'options'}
      onClick$={() => {
        data.selectedOutputDetail = 'options';
      }}
    />
  );
};
