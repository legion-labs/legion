import apiCodegen from "@lgn/vite-plugin-api-codegen";

async function build() {
  await apiCodegen({
    path: "../../crates/lgn-streamer/apis",
    apiNames: ["streaming"],
    withPackageJson: true,
    aliasMappings: {
      "../../crates/lgn-governance/apis/space.yaml": "Space",
      "../../crates/lgn-governance/apis/workspace.yaml": "Workspace",
    },
  }).buildStart();
}

build().catch(console.error);
