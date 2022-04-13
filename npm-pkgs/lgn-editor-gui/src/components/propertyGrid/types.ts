import type { ResourceProperty } from "@/lib/propertyGrid";

/** Add a sub property to a vector event payload */
export type AddVectorSubPropertyEvent = {
  path: string;
  index: number;
  property: ResourceProperty;
};

/** Remove a sub property from a vector event payload */
export type RemoveVectorSubPropertyEvent = {
  path: string;
  index: number;
};
