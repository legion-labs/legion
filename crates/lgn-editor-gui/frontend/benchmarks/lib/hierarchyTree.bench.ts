import { suite, add, cycle, complete } from "benny";

import { Entries } from "@/lib/hierarchyTree";
import resources from "../resources/resourcesResponse.json";

suite(
  "Entries.unflatten",
  add("unflatten entries", () => {
    Entries.unflatten(resources);
  }),
  cycle(),
  complete()
);
