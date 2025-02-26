#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Example1Snapshot {
    pub renderHeader_div_onClick_fV2uzAL99u4: String,
    pub renderHeader_zBbHWn4e8Cg: String,
    pub renderHeader_component_U6Kkv07sbpQ: String,
    pub body: String,
}

impl Example1Snapshot {
    pub fn new() -> Self {
        let renderHeader_div_onClick_fV2uzAL99u4 =
            r#"export const renderHeader_div_onClick_fV2uzAL99u4 = (ctx) => console.log(ctx);
export { _hW } from "@builder.io/qwik";"#
                .trim()
                .to_string();

        let renderHeader_zBbHWn4e8Cg = r#"import { qrl } from "@builder.io/qwik";
export const renderHeader_zBbHWn4e8Cg = () => {
return <div onClick={qrl(() => import("./test.tsx_renderHeader_div_onClick_fV2uzAL99u4"), "renderHeader_div_onClick_fV2uzAL99u4")} />;
};
export { _hW } from "@builder.io/qwik";"#.trim().to_string();

        let renderHeader_component_U6Kkv07sbpQ = r#"import { qrl } from "@builder.io/qwik";
export const renderHeader_component_U6Kkv07sbpQ = () => {
	console.log("mount");
	return render;
};
export { _hW } from "@builder.io/qwik";"#
            .trim()
            .to_string();

        let body = String::new();

        Example1Snapshot {
            renderHeader_div_onClick_fV2uzAL99u4,
            renderHeader_zBbHWn4e8Cg,
            renderHeader_component_U6Kkv07sbpQ,
            body,
        }
    }
}

impl Default for Example1Snapshot {
    fn default() -> Self {
        Self::new()
    }
}
