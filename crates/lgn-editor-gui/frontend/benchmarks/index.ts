import { run } from "./benchmark";

// Import a .bench file here so it can be executed by the `pnpm benchmark` command
import { bigResourcesSuite, resourcesSuite } from "./lib/hierarchyTree.bench";

run([bigResourcesSuite, resourcesSuite]);
