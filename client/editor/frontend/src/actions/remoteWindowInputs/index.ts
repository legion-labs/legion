import log from "@/lib/log";
import { KeyCode, fromBrowserKey as keyCodeFromBrowserKey } from "./keys";

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
export type ElementState = "Pressed" | "Released";

type Type<Type extends string> = { type: Type };

/** A mouse button input */
export type MouseButtonInput = Type<"MouseButtonInput"> & {
  /** The mouse button (typically Left/Middle/Right) */
  button: MouseButton;
  /** The mouse button state Pressed/Released */
  state: ElementState;
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

export type TouchPhase = "Started" | "Moved" | "Ended" | "Cancelled";

// As per https://developer.mozilla.org/en-US/docs/Web/API/Touch/force,
// it seems that the whole notion of "Calibrated" force doesn't exist
// at all in the browsers, so we send only normalized force
export type ForceTouch = {
  Normalized: /* f64 */ number;
};

export type TouchInput = Type<"TouchInput"> & {
  phase: TouchPhase;
  position: Vec2;
  /** Describes how hard the screen was pressed. May be `None` if the platform
   * does not support pressure sensitivity.
   *
   * ## Platform-specific
   *
   * - Only available on **iOS** 9.0+ and **Windows** 8+.
   */
  force: ForceTouch | null;
  /** Unique identifier of a finger.*/
  id: /* u64 */ number;
};

export type KeyboardInput = Type<"KeyboardInput"> & {
  // Browser events don't contain the scan code
  scan_code: /* u32 */ 0;
  key_code: KeyCode;
  state: ElementState;
};

/** The Input type union */
export type Input =
  | MouseButtonInput
  | MouseMotion
  | MouseWheel
  | TouchInput
  | KeyboardInput;

/** A function passed to the `remotedWindowEvents` action that will be called when an event is dispatched */
export type Listener = (input: Input) => void;

export type Options = {
  isFocused: boolean;
  listener: Listener;
};

type State = {
  mouseState: ElementState;
  /** Where the index is the Touch id.
   * We use an object of `null` value instead of an array
   * so that it's easier and faster to lookup for ids and
   * to delete the touch action that's not active anymore
   */
  activeTouches: Record<number, null>;
  /**
   * Where the index is the `KeyCode`.
   */
  activeKeys: Record<string, null>;
  previousMousePosition: Vec2 | null;
  isFocused: boolean;
};

function createEvents(state: State, element: HTMLElement, listener: Listener) {
  function getCurrentMousePosition({
    clientX,
    clientY,
  }: {
    clientX: number;
    clientY: number;
  }): Vec2 {
    const { left, top } = element.getBoundingClientRect();

    return [clientX - left, clientY - top];
  }

  function onContextMenu(event: MouseEvent) {
    if (!isNode(event.target) || !element.contains(event.target)) {
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    return false;
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
    const previousMouseState = state.mouseState;

    state.mouseState = "Released";

    // This means the mouse up event wasn't initiated by a mouse down
    // in the remote window, we don't need to send the input to the server
    if (previousMouseState !== "Pressed") {
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

    event.preventDefault();

    let unit: MouseScrollUnit;

    // https://developer.mozilla.org/en-US/docs/Web/API/WheelEvent/deltaMode
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

  function onTouchStart(event: TouchEvent) {
    if (!isNode(event.target) || !element.contains(event.target)) {
      return;
    }

    event.preventDefault();

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      state.activeTouches[changedTouch.identifier] = null;

      const touchInput: TouchInput = {
        type: "TouchInput",
        phase: "Started",
        force: { Normalized: changedTouch.force },
        // The identifier id unique for each touch event,
        // making it unique for the finger.
        id: changedTouch.identifier,
        position: getCurrentMousePosition(changedTouch),
      };

      log.debug(logLabel, log.json`Touch input ${touchInput}`);

      listener(touchInput);
    }
  }

  function onTouchMove(event: TouchEvent) {
    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      if (
        (!isNode(event.target) || !element.contains(event.target)) &&
        !(changedTouch.identifier in state.activeTouches)
      ) {
        continue;
      }

      const touchInput: TouchInput = {
        type: "TouchInput",
        phase: "Moved",
        force: { Normalized: changedTouch.force },
        // The identifier id unique for each touch event,
        // making it unique for the finger.
        id: changedTouch.identifier,
        position: getCurrentMousePosition(changedTouch),
      };

      log.debug(logLabel, log.json`Touch input ${touchInput}`);

      listener(touchInput);
    }
  }

  function onTouchEnd(event: TouchEvent) {
    let defaultPrevented = false;

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      // This means the touch end event wasn't initiated by a touch start
      // in the remote window, we don't need to send the input to the server
      if (!(changedTouch.identifier in state.activeTouches)) {
        continue;
      }

      delete state.activeTouches[changedTouch.identifier];

      if (!defaultPrevented) {
        event.preventDefault();
        defaultPrevented = true;
      }

      const touchInput: TouchInput = {
        type: "TouchInput",
        phase: "Ended",
        force: { Normalized: changedTouch.force },
        // The identifier id unique for each touch event,
        // making it unique for the finger.
        id: changedTouch.identifier,
        position: getCurrentMousePosition(changedTouch),
      };

      log.debug(logLabel, log.json`Touch input ${touchInput}`);

      listener(touchInput);
    }
  }

  function onTouchCancel(event: TouchEvent) {
    let defaultPrevented = false;

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      // This means the touch end event wasn't initiated by a touch start
      // in the remote window, we don't need to send the input to the server
      if (!(changedTouch.identifier in state.activeTouches)) {
        continue;
      }

      delete state.activeTouches[changedTouch.identifier];

      if (!defaultPrevented) {
        event.preventDefault();
        defaultPrevented = true;
      }

      const touchInput: TouchInput = {
        type: "TouchInput",
        phase: "Cancelled",
        force: { Normalized: changedTouch.force },
        // The identifier id unique for each touch event,
        // making it unique for the finger.
        id: changedTouch.identifier,
        position: getCurrentMousePosition(changedTouch),
      };

      log.debug(logLabel, log.json`Touch input ${touchInput}`);

      listener(touchInput);
    }
  }

  function onKeyDown(event: KeyboardEvent) {
    if (!state.isFocused) {
      return;
    }

    const key = keyCodeFromBrowserKey(event.key);

    // We don't report unknown keys or keys that are being pressed already
    if (!key || key in state.activeKeys) {
      return;
    }

    event.preventDefault();

    state.activeKeys[key] = null;

    const keyboardInput: KeyboardInput = {
      type: "KeyboardInput",
      key_code: key,
      scan_code: 0,
      state: "Pressed",
    };

    log.debug(logLabel, log.json`Keyboard input ${keyboardInput}`);

    listener(keyboardInput);
  }

  function onKeyUp(event: KeyboardEvent) {
    if (!state.isFocused) {
      return;
    }

    const key = keyCodeFromBrowserKey(event.key);

    if (!key || !(key in state.activeKeys)) {
      return;
    }

    event.preventDefault();

    delete state.activeKeys[key];

    const keyboardInput: KeyboardInput = {
      type: "KeyboardInput",
      key_code: key,
      scan_code: 0,
      state: "Released",
    };

    log.debug(logLabel, log.json`Keyboard input ${keyboardInput}`);

    listener(keyboardInput);
  }

  return {
    onContextMenu,
    onMouseDown,
    onMouseMove,
    onMouseUp,
    onWheel,
    onTouchStart,
    onTouchMove,
    onTouchEnd,
    onTouchCancel,
    onKeyDown,
    onKeyUp,
  };
}

export default function remoteWindowEvents(
  element: HTMLElement,
  { isFocused, listener }: Options
) {
  element.style.touchAction = "none";

  const state: State = {
    mouseState: "Released",
    activeTouches: {},
    activeKeys: {},
    previousMousePosition: null,
    isFocused,
  };

  const {
    onContextMenu,
    onMouseDown,
    onMouseMove,
    onMouseUp,
    onWheel,
    onTouchStart,
    onTouchMove,
    onTouchEnd,
    onTouchCancel,
    onKeyDown,
    onKeyUp,
  } = createEvents(state, element, listener);

  // Global listeners, useful when an event occurs outside
  // the remote window but still has to be sent to the server
  window.addEventListener("mousemove", onMouseMove);

  window.addEventListener("mouseup", onMouseUp);

  window.addEventListener("touchmove", onTouchMove);

  window.addEventListener("touchend", onTouchEnd);

  window.addEventListener("touchcancel", onTouchCancel);

  window.addEventListener("keydown", onKeyDown);

  window.addEventListener("keyup", onKeyUp);

  // Element listeners
  element.addEventListener("contextmenu", onContextMenu);

  element.addEventListener("mousedown", onMouseDown);

  element.addEventListener("wheel", onWheel, { passive: false });

  element.addEventListener("touchstart", onTouchStart);

  return {
    update({ isFocused }: Options) {
      state.isFocused = isFocused;
    },
    destroy() {
      window.removeEventListener("mousemove", onMouseMove);

      window.removeEventListener("mouseup", onMouseUp);

      window.removeEventListener("touchmove", onTouchMove);

      window.removeEventListener("touchend", onTouchEnd);

      window.removeEventListener("touchcancel", onTouchCancel);

      window.removeEventListener("keydown", onKeyDown);

      window.removeEventListener("keyup", onKeyUp);

      element.removeEventListener("contextmenu", onContextMenu);

      element.removeEventListener("mousedown", onMouseDown);

      element.removeEventListener("wheel", onWheel);

      element.removeEventListener("touchstart", onTouchStart);
    },
  };
}
