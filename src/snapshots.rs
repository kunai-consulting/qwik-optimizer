use crate::component::normalize_test_output;

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

        let body =r#"import { qrl } from "@builder.io/qwik";
import { $, component, onRender } from "@builder.io/qwik";
export const renderHeader = qrl(() => import("./test.tsx_renderHeader_zBbHWn4e8Cg"), "renderHeader_zBbHWn4e8Cg");
const renderHeader = component(qrl(() => import("./test.tsx_renderHeader_component_U6Kkv07sbpQ"), "renderHeader_component_U6Kkv07sbpQ"));"#
            .trim()
            .to_string();

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Example2Snapshot {
    pub Header_component_div_onClick_i7ekvWH367g: String,
    pub Header_component_J4uyIhaBNR4: String,
    pub body: String
}

impl Example2Snapshot {
    pub fn new() -> Self {
        let Header_component_div_onClick_i7ekvWH367g = normalize_test_output(
            r#"export const Header_component_div_onClick_i7ekvWH3674 = (ctx) => console.log(ctx);
export { _hW } from "@builder.io/qwik";"#,
        );

        let Header_component_J4uyIhaBNR4 = normalize_test_output(
            r#"import { qrl } from "@builder.io/qwik";
export const Header_component_J4uyIhaBNR4 = () => {
    console.log("mount");
    return <div onClick={/*#__PURE__*/ qrl(() => import("./test.tsx_Header_component_div_onClick_i7ekvWH3674"), "Header_component_div_onClick_i7ekvWH3674")} />;
};"#,
        );
        
        let body = normalize_test_output(
            r#"import { componentQrl, qrl } from "@builder.io/qwik";
export const Header = /*#__PURE__*/ componentQrl(/*#__PURE__*/ qrl(()=>import("./test.tsx_Header_component_J4uyIhaBNR4"), "Header_component_J4uyIhaBNR4"));"#);
        
        
        Example2Snapshot {
            Header_component_div_onClick_i7ekvWH367g,
            Header_component_J4uyIhaBNR4,
            body,
        }
    }
}

impl Default for Example2Snapshot {
    fn default() -> Self {
        Self::new()
    }
}
