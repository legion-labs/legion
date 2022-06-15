import apiCodegen from "@lgn/vite-plugin-api-codegen";

async function build() {
  await Promise.all([
    apiCodegen({
      path: "../../crates/lgn-streamer/apis",
      apiNames: ["streaming"],
      withPackageJson: true,
      aliasMappings: {
        "../../crates/lgn-governance/apis/space.yaml": "Space",
        "../../crates/lgn-governance/apis/workspace.yaml": "Workspace",
      },
      filename: "streaming",
    }).buildStart(),
    apiCodegen({
      path: "../../crates/lgn-log/apis",
      apiNames: ["log"],
      withPackageJson: true,
      aliasMappings: {
        "../../crates/lgn-governance/apis/space.yaml": "Space",
        "../../crates/lgn-governance/apis/workspace.yaml": "Workspace",
      },
      filename: "log",
    }).buildStart(),
  ]);
}

build().catch(console.error);
