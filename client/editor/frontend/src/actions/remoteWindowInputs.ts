import log from "@/lib/log";

const logLabel = "remote window inputs";

export type Vec2 = [x: number, y: number];

function keyToMouseButton(key: number): MouseButton {
  // https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
  switch (key) {
    case 0:
      return "Left";
    case 1:
      return "Middle";
    case 2:
      return "Right";
    // TODO: A bit unsure about this one
    default:
      return { Other: key };
  }
}

export type MouseButton =
  | "Left"
  | "Middle"
  | "Right"
  | { Other: /* u16 */ number };

/**
 * "Pressed" on cursor press/when the button is pressed,
 * "Released" by default and when the cursor button has been released
 */
export type MouseState = "Pressed" | "Released";

type InputBase<Type extends string> = { type: Type };

/** A mouse button input */
export type MouseButtonInput = InputBase<"MouseButtonInput"> & {
  /** The mouse button (typically Left/Middle/Right) */
  button: MouseButton;
  /** The mouse button state Pressed/Released */
  state: MouseState;
  /** The mouse cursor position */
  pos: Vec2;
};

/** Represents a cursor move input, the last known cursor position and the current cursor position are included */
export type CursorMoved = InputBase<"CursorMoved"> & {
  /** The difference between the last known position and the current one */
  delta: Vec2;
};

/** The Input type union */
export type Input = MouseButtonInput | CursorMoved;

/** A function passed to the `remotedWindowEvents` action that will be called when an event is dispatched */
export type Listener = (input: Input) => void;

type State = {
  mouseState: MouseState;
  previousMousePosition: Vec2 | null;
};

function isNode(element: unknown): element is Node {
  return element instanceof Node;
}

function createEvents(
  state: State,
  element: HTMLElement,
  listener: Listener = () => {
    // No op
  }
) {
  function getCurrentMousePosition(event: MouseEvent): Vec2 {
    const { left, top } = element.getBoundingClientRect();

    return [event.clientX - left, event.clientY - top];
  }

  function onMouseDown(event: MouseEvent) {
    if (!isNode(event.target) || !element.contains(event.target)) {
      return;
    }

    state.mouseState = "Pressed";

    const mouseButtonInput: MouseButtonInput = {
      type: "MouseButtonInput",
      button: keyToMouseButton(event.button),
      state: "Pressed",
      pos: getCurrentMousePosition(event),
    };

    log.debug(logLabel, log.json`Mouse button input ${mouseButtonInput}`);

    listener(mouseButtonInput);
  }

  function onMouseUp(event: MouseEvent) {
    state.mouseState = "Released";

    if (!isNode(event.target) || !element.contains(event.target)) {
      return;
    }

    const mouseButtonInput: MouseButtonInput = {
      type: "MouseButtonInput",
      button: keyToMouseButton(event.button),
      state: "Released",
      pos: getCurrentMousePosition(event),
    };

    log.debug(logLabel, log.json`Mouse button input ${mouseButtonInput}`);

    listener(mouseButtonInput);
  }

  function onMouseMove(event: MouseEvent) {
    if (
      (!isNode(event.target) || !element.contains(event.target)) &&
      state.mouseState !== "Pressed"
    ) {
      return;
    }

    const previousMousePosition = state.previousMousePosition ?? [0, 0];

    const currentMousePosition = getCurrentMousePosition(event);

    state.previousMousePosition = currentMousePosition;

    const mouseMoveEvent: CursorMoved = {
      type: "CursorMoved",
      delta: [
        currentMousePosition[0] - previousMousePosition[0],
        currentMousePosition[1] - previousMousePosition[1],
      ],
    };

    log.debug(logLabel, log.json`Cursor moved ${mouseMoveEvent}`);

    listener(mouseMoveEvent);
  }

  return { onMouseDown, onMouseMove, onMouseUp };
}

export default function remoteWindowEvents(
  element: HTMLElement,
  listener?: Listener
) {
  const state: State = {
    mouseState: "Released",
    previousMousePosition: null,
  };

  const { onMouseDown, onMouseMove, onMouseUp } = createEvents(
    state,
    element,
    listener
  );

  window.addEventListener("mousedown", onMouseDown);

  window.addEventListener("mouseup", onMouseUp);

  window.addEventListener("mousemove", onMouseMove);

  return {
    destroy() {
      window.removeEventListener("mousedown", onMouseDown);

      window.removeEventListener("mouseup", onMouseUp);

      window.removeEventListener("mousemove", onMouseMove);
    },
  };
}
