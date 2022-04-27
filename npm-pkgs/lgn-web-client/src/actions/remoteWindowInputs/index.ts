import log from "../../lib/log";
import type { GamepadAxisType, GamepadButtonType } from "./gamepad";
import { fromGamepadAxisIndex, fromGamepadButtonIndex } from "./gamepad";
import type { KeyCode } from "./keys";
import { fromBrowserKey as keyCodeFromBrowserKey } from "./keys";
import type { GamepadButtonType, GamepadAxisType } from "./gamepad";

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
  /** The current cursor position */
  current: Vec2;
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

export type GamepadConnection = Type<"GamepadConnection"> & {
  pad_id: number;
};

export type GamepadDisconnection = Type<"GamepadDisconnection"> & {
  pad_id: number;
};

export type GamepadButtonChange = Type<"GamepadButtonChange"> & {
  pad_id: number;
  button: GamepadButtonType;
  value: number;
};

export type GamepadAxisChange = Type<"GamepadAxisChange"> & {
  pad_id: number;
  axis: GamepadAxisType;
  value: number;
};

/** The Input type union */
export type RemoteWindowInput =
  | MouseButtonInput
  | MouseMotion
  | MouseWheel
  | TouchInput
  | KeyboardInput
  | GamepadConnection
  | GamepadDisconnection
  | GamepadButtonChange
  | GamepadAxisChange;

/** A function passed to the `remotedWindowEvents` action that will be called when an event is dispatched */
export type Listener = (input: RemoteWindowInput) => void;

type State = {
  mouseState: ElementState;
  /** Contains the Touch id */
  activeTouches: Set<number>;
  /** Contains the `KeyCode` */
  activeKeys: Set<string>;
  previousMousePosition: Vec2 | null;
  gamepads: (Gamepad | null)[];
  animationFrame: number | null;
};

function createEvents(state: State, element: HTMLElement, onInput: Listener) {
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
    event.preventDefault();
    event.stopPropagation();

    return false;
  }

  function onMouseDown(event: MouseEvent) {
    state.mouseState = "Pressed";

    const mouseButtonInput: MouseButtonInput = {
      type: "MouseButtonInput",
      button: keyToMouseButton(event.button),
      state: "Pressed",
      pos: getCurrentMousePosition(event),
    };

    log.debug(logLabel, log.json`Mouse button input ${mouseButtonInput}`);

    onInput(mouseButtonInput);
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

    onInput(mouseButtonInput);
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
      current: currentMousePosition,
      delta: [
        currentMousePosition[0] - previousMousePosition[0],
        currentMousePosition[1] - previousMousePosition[1],
      ],
    };

    log.debug(logLabel, log.json`Cursor moved ${mouseMotion}`);

    onInput(mouseMotion);
  }

  function onWheel(event: WheelEvent) {
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

    onInput(wheelInput);
  }

  function onTouchStart(event: TouchEvent) {
    event.preventDefault();

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      state.activeTouches.add(changedTouch.identifier);

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

      onInput(touchInput);
    }
  }

  function onTouchMove(event: TouchEvent) {
    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      if (!state.activeTouches.has(changedTouch.identifier)) {
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

      onInput(touchInput);
    }
  }

  function onTouchEnd(event: TouchEvent) {
    let defaultPrevented = false;

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      // This means the touch end event wasn't initiated by a touch start
      // in the remote window, we don't need to send the input to the server
      if (!state.activeTouches.has(changedTouch.identifier)) {
        continue;
      }

      state.activeTouches.delete(changedTouch.identifier);

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

      onInput(touchInput);
    }
  }

  function onTouchCancel(event: TouchEvent) {
    let defaultPrevented = false;

    for (let i = 0; i < event.changedTouches.length; i++) {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const changedTouch = event.changedTouches.item(i)!;

      // This means the touch end event wasn't initiated by a touch start
      // in the remote window, we don't need to send the input to the server
      if (!state.activeTouches.has(changedTouch.identifier)) {
        continue;
      }

      state.activeTouches.delete(changedTouch.identifier);

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

      onInput(touchInput);
    }
  }

  function onKeyDown(event: KeyboardEvent) {
    const key = keyCodeFromBrowserKey(event.key, event.location);

    // We don't report unknown keys
    if (!key) {
      return;
    }

    event.preventDefault();

    state.activeKeys.add(key);

    const keyboardInput: KeyboardInput = {
      type: "KeyboardInput",
      // eslint-disable-next-line camelcase
      key_code: key,
      // eslint-disable-next-line camelcase
      scan_code: 0,
      state: "Pressed",
    };

    log.debug(logLabel, log.json`Keyboard input ${keyboardInput}`);

    onInput(keyboardInput);
  }

  function onKeyUp(event: KeyboardEvent) {
    const key = keyCodeFromBrowserKey(event.key, event.location);

    if (!key || !state.activeKeys.has(key)) {
      return;
    }

    event.preventDefault();

    state.activeKeys.delete(key);

    const keyboardInput: KeyboardInput = {
      type: "KeyboardInput",
      // eslint-disable-next-line camelcase
      key_code: key,
      // eslint-disable-next-line camelcase
      scan_code: 0,
      state: "Released",
    };

    log.debug(logLabel, log.json`Keyboard input ${keyboardInput}`);

    onInput(keyboardInput);
  }

  function onGamepadConnected(event: GamepadEvent) {
    const gamepadConnection: GamepadConnection = {
      type: "GamepadConnection",
      // eslint-disable-next-line camelcase
      pad_id: event.gamepad.index,
    };

    log.debug(logLabel, log.json`Gamepad connection ${gamepadConnection}`);

    onInput(gamepadConnection);
  }

  function onGamepadDisconnected(event: GamepadEvent) {
    const gamepadDisconnection: GamepadDisconnection = {
      type: "GamepadDisconnection",
      // eslint-disable-next-line camelcase
      pad_id: event.gamepad.index,
    };

    log.debug(
      logLabel,
      log.json`Gamepad disconnection ${gamepadDisconnection}`
    );

    onInput(gamepadDisconnection);
  }

  function scanGamepads() {
    if (state.gamepads !== null) {
      const gamepads = navigator.getGamepads();

      for (
        let gamepadIndex = 0;
        gamepadIndex < gamepads.length;
        gamepadIndex++
      ) {
        const oldGamepad = state.gamepads[gamepadIndex];
        const newGamepad = gamepads[gamepadIndex];

        if (newGamepad?.connected) {
          for (
            let buttonIndex = 0;
            buttonIndex < newGamepad.buttons.length;
            buttonIndex++
          ) {
            const newButton = newGamepad.buttons[buttonIndex];
            let valueChanged = false;
            let pressedChange = false;

            if (oldGamepad !== null) {
              const oldButton = oldGamepad.buttons[buttonIndex];

              valueChanged = newButton.value !== oldButton.value;
              pressedChange = newButton.pressed !== oldButton.pressed;
            } else {
              valueChanged = true;
              pressedChange = true;
            }

            if (valueChanged || pressedChange) {
              const button = fromGamepadButtonIndex(buttonIndex);

              if (button !== null) {
                if (valueChanged) {
                  const gamepadButtonChange: GamepadButtonChange = {
                    type: "GamepadButtonChange",
                    // eslint-disable-next-line camelcase
                    pad_id: gamepadIndex,
                    button: button,
                    value: newButton.value,
                  };

                  log.debug(
                    logLabel,
                    log.json`Gamepad button change ${gamepadButtonChange}`
                  );

                  onInput(gamepadButtonChange);
                } else if (pressedChange) {
                  const value = newButton.pressed ? 1.0 : 0.0;
                  const gamepadButtonChange: GamepadButtonChange = {
                    type: "GamepadButtonChange",
                    // eslint-disable-next-line camelcase
                    pad_id: gamepadIndex,
                    button: button,
                    value: value,
                  };

                  log.debug(
                    logLabel,
                    log.json`Gamepad button change ${gamepadButtonChange}`
                  );

                  onInput(gamepadButtonChange);
                }
              }
            }
          }

          for (
            let axisIndex = 0;
            axisIndex < newGamepad.axes.length;
            axisIndex++
          ) {
            const newAxis = newGamepad.axes[axisIndex];
            let valueChanged = false;

            if (oldGamepad !== null) {
              const oldAxis = oldGamepad.axes[axisIndex];

              valueChanged = newAxis !== oldAxis;
            } else {
              valueChanged = true;
            }

            if (valueChanged) {
              const axisDescription = fromGamepadAxisIndex(axisIndex);

              if (axisDescription !== null) {
                const gamepadAxisChange: GamepadAxisChange = {
                  type: "GamepadAxisChange",
                  // eslint-disable-next-line camelcase
                  pad_id: gamepadIndex,
                  axis: axisDescription.axisType,
                  value: axisDescription.inverted ? -newAxis : newAxis,
                };

                log.debug(
                  logLabel,
                  log.json`Gamepad axis change ${gamepadAxisChange}`
                );

                onInput(gamepadAxisChange);
              }
            }
          }
        }
      }

      state.gamepads = gamepads;
    }

    state.animationFrame = requestAnimationFrame(scanGamepads);
  }

  state.animationFrame = requestAnimationFrame(scanGamepads);

  return {
    // Window listeners, useful when an event occurs outside
    // the remote window and still has to be sent to the server
    window: {
      onMouseMove,
      onMouseUp,
      onGamepadConnected,
      onGamepadDisconnected,
    },
    // Listeners attached to the element
    element: {
      onContextMenu,
      onMouseDown,
      onWheel,
      onTouchStart,
      onTouchMove,
      onTouchEnd,
      onTouchCancel,
      onKeyDown,
      onKeyUp,
    },
  };
}

/**
 * The `removeWindowInputs` action will automatically attach all keyboard, touch,
 * and mouse input events, and turn them into proper lgn-input's inputs.
 *
 * All inputs will be passed down to the provided `onInput` function.
 */
export default function remoteWindowInputs(
  element: HTMLElement,
  onInput: Listener
) {
  element.tabIndex = -1;

  element.style.touchAction = "none";
  element.style.outline = "none";

  const state: State = {
    mouseState: "Released",
    activeTouches: new Set(),
    activeKeys: new Set(),
    previousMousePosition: null,
    gamepads: [],
    animationFrame: null,
  };

  const listeners = createEvents(state, element, onInput);

  window.addEventListener("mousemove", listeners.window.onMouseMove);

  window.addEventListener("mouseup", listeners.window.onMouseUp);

  window.addEventListener(
    "gamepadconnected",
    listeners.window.onGamepadConnected
  );

  window.addEventListener(
    "gamepaddisconnected",
    listeners.window.onGamepadDisconnected
  );

  element.addEventListener("contextmenu", listeners.element.onContextMenu);

  element.addEventListener("mousedown", listeners.element.onMouseDown);

  element.addEventListener("wheel", listeners.element.onWheel, {
    passive: false,
  });

  element.addEventListener("touchstart", listeners.element.onTouchStart);

  element.addEventListener("touchmove", listeners.element.onTouchMove);

  element.addEventListener("touchend", listeners.element.onTouchEnd);

  element.addEventListener("touchcancel", listeners.element.onTouchCancel);

  element.addEventListener("keydown", listeners.element.onKeyDown);

  element.addEventListener("keyup", listeners.element.onKeyUp);

  return {
    destroy() {
      window.removeEventListener("mousemove", listeners.window.onMouseMove);

      window.removeEventListener("mouseup", listeners.window.onMouseUp);

      element.removeEventListener(
        "contextmenu",
        listeners.element.onContextMenu
      );

      element.removeEventListener("mousedown", listeners.element.onMouseDown);

      element.removeEventListener("wheel", listeners.element.onWheel);

      element.removeEventListener("touchstart", listeners.element.onTouchStart);

      element.removeEventListener("touchmove", listeners.element.onTouchMove);

      element.removeEventListener("touchend", listeners.element.onTouchEnd);

      element.removeEventListener(
        "touchcancel",
        listeners.element.onTouchCancel
      );

      element.removeEventListener("keydown", listeners.element.onKeyDown);

      element.removeEventListener("keyup", listeners.element.onKeyUp);
    },
  };
}
