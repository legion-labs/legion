/**
 * Exports a properly typed `contextMenu` store.
 */

import buildContextMenuStore from "@lgn/frontend/src/stores/contextMenu";

export type ContextMenuName = "resource";

export default buildContextMenuStore<ContextMenuName>();
