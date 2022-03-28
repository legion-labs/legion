// This orchestrator manipulates several stores related to the current selection(s),
// that is, the element(s) selected in the viewport/resource browser/scene explorer.
//
// This orchestrator doesn't own any stores.

import { get } from "svelte/store";
import { initMessageStream as initMessageStreamApi } from "@/api";
import {
  resourceEntries,
  currentResourceDescriptionEntry,
} from "./resourceBrowserEntries";
import { MessageType } from "@lgn/proto-editor/dist/editor";
import { fetchCurrentResourceDescription } from "@/orchestrators/currentResource";
import { isEntry } from "@/lib/hierarchyTree";

export async function initMessageStream() {
  const messageStream = await initMessageStreamApi();

  messageStream.subscribe(({ lagging, message }) => {
    if (typeof lagging === "number") {
      // TODO: Handle lagging messages

      return;
    }

    if (message) {
      switch (message.msgType) {
        case MessageType.SelectionChanged: {
          const resourceIds: string[] = JSON.parse(message.payload);

          // TODO: Catch error
          // TODO: Support multi-select (remove slice)
          if (!resourceIds.length) {
            currentResourceDescriptionEntry.set(null);

            return;
          }

          fetchCurrentResourceDescription(resourceIds[0], {
            notifySelection: false,
          });

          const selectedEntry = get(resourceEntries).find(
            (entry) => isEntry(entry) && resourceIds.includes(entry.item.id)
          );

          currentResourceDescriptionEntry.set(selectedEntry);
        }
      }
    }
  });
}
