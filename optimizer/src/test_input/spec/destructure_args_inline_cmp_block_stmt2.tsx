export default (props: { data: any }) => {
  const { data } = props;
  return (
    <div
      data-is-active={data.selectedOutputDetail === 'options'}
      onClick$={() => {
        data.selectedOutputDetail = 'options';
      }}
    />
  );
};
