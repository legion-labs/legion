// TODO: Uses the types present here:
// https://github.com/sveltejs/svelte/pull/7121/files
// At the time of writing the types are not yet available in Svelte
// So we just copy paste them
export interface ActionReturn<Parameter = unknown> {
  update?: (parameter: Parameter) => void;
  destroy?: () => void;
}

export interface Action<Element = HTMLElement, Parameter = unknown> {
  <Node extends Element>(
    node: Node,
    parameter: Parameter
  ): void | ActionReturn<Parameter>;
}

export function nullable<Element = HTMLElement, Parameter = unknown>(
  action: Action<Element, Parameter>
): Action<Element, Parameter | null> {
  return function (element: Element, parameter: Parameter | null) {
    // We only early return if the parameter is null _not_ undefined
    if (parameter === null) {
      return;
    }

    return action(element, parameter) as ActionReturn<Parameter | null> | void;
  };
}
