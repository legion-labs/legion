import log from "@/lib/log";

const logLabel = "remote window inputs";

export type Vec2 = [x: /* f32 */ number, y: /* f32 */ number];

/** Takes an `event.button` "key" and return a proper `MouseButton` value */
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

/** Takes anything and returns `true` if the passed object is a `Node` */
function isNode(element: unknown): element is Node {
  return element instanceof Node;
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

type Type<Type extends string> = { type: Type };

/** A mouse button input */
export type MouseButtonInput = Type<"MouseButtonInput"> & {
  /** The mouse button (typically Left/Middle/Right) */
  button: MouseButton;
  /** The mouse button state Pressed/Released */
  state: MouseState;
  /** The mouse cursor position */
  pos: Vec2;
};

/** Represents a cursor move input, the last known cursor position and the current cursor position are included */
export type MouseMotion = Type<"MouseMotion"> & {
  /** The difference between the last known position and the current one */
  delta: Vec2;
};

export type MouseScrollUnit = "Line" | "Pixel";

export type MouseWheel = Type<"MouseWheel"> & {
  unit: MouseScrollUnit;
  x: /* f32 */ number;
  y: /* f32 */ number;
};

/** The Input type union */
export type Input = MouseButtonInput | MouseMotion | MouseWheel;

/** A function passed to the `remotedWindowEvents` action that will be called when an event is dispatched */
export type Listener = (input: Input) => void;

type State = {
  mouseState: MouseState;
  previousMousePosition: Vec2 | null;
};

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

    const mouseMotion: MouseMotion = {
      type: "MouseMotion",
      delta: [
        currentMousePosition[0] - previousMousePosition[0],
        currentMousePosition[1] - previousMousePosition[1],
      ],
    };

    log.debug(logLabel, log.json`Cursor moved ${mouseMotion}`);

    listener(mouseMotion);
  }

  function onWheel(event: WheelEvent) {
    if (!isNode(event.target) || !element.contains(event.target)) {
      return;
    }

    let unit: MouseScrollUnit;

    switch (event.deltaMode) {
      case WheelEvent.DOM_DELTA_PIXEL: {
        unit = "Pixel";

        break;
      }
      case WheelEvent.DOM_DELTA_LINE: {
        unit = "Line";

        break;
      }
      case WheelEvent.DOM_DELTA_PAGE: {
        log.error(
          logLabel,
          "Mouse wheel delta mode was specified in page which is not supported"
        );

        return;
      }
      default: {
        log.error(
          logLabel,
          `Unknown mouse wheel delta mode ${event.deltaMode}`
        );

        return;
      }
    }

    const wheelInput: MouseWheel = {
      type: "MouseWheel",
      unit,
      x: event.deltaX,
      y: event.deltaY,
    };

    log.debug(logLabel, log.json`Mouse wheel ${wheelInput}`);

    listener(wheelInput);
  }

  return { onMouseDown, onMouseMove, onMouseUp, onWheel };
}

export default function remoteWindowEvents(
  element: HTMLElement,
  listener?: Listener
) {
  const state: State = {
    mouseState: "Released",
    previousMousePosition: null,
  };

  const { onMouseDown, onMouseMove, onMouseUp, onWheel } = createEvents(
    state,
    element,
    listener
  );

  window.addEventListener("mousedown", onMouseDown);

  window.addEventListener("mouseup", onMouseUp);

  window.addEventListener("mousemove", onMouseMove);

  window.addEventListener("wheel", onWheel);

  return {
    destroy() {
      window.removeEventListener("mousedown", onMouseDown);

      window.removeEventListener("mouseup", onMouseUp);

      window.removeEventListener("mousemove", onMouseMove);

      window.removeEventListener("wheel", onWheel);
    },
  };
}
