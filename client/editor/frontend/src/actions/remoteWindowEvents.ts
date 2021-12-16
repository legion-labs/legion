import log from "@/lib/log";

/** A mouse position */
export type Position = { x: number; y: number };

// https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
/** The mouse buttons map */
const mouseButtons = {
  /** Usually the left button */
  main: 0,
  /** Usually the wheel/middle button */
  auxiliary: 1,
  /** Usually the right button */
  secondary: 2,
};

type EventBase<Type extends string> = { type: Type };

/**
 * "down" on mouse press/when the button is pressed, "up" by default and when the mouse button has been released
 */
type MouseStatus = "up" | "down";

/** Represents a click event, the mouse position is included */
export type ClickEvent = EventBase<"click"> & { position: Position };

/** Represents a mouse move event, the last known mouse position and the current mouse position are included */
export type MouseMoveEvent = EventBase<"mousemove"> & {
  /** The last known mouse position */
  from: Position;
  /** The current mouse position */
  to: Position;
};

/** The Event type union */
export type Event = ClickEvent | MouseMoveEvent;

/** A function passed to the `remotedWindowEvents` action that will be called when an event is dispatched */
export type Listener = (event: Event) => void;

type State = {
  mouseStatus: MouseStatus;
  mouseIsMoving: boolean;
  previousMousePosition: Position | null;
};

function createEvents(state: State, element: HTMLElement, listener?: Listener) {
  function onMouseDown(event: MouseEvent) {
    if (
      event.button !== mouseButtons.main ||
      !event.target ||
      !element.contains(event.target as Node)
    ) {
      return;
    }

    state.mouseStatus = "down";
  }

  function onMouseUp(event: MouseEvent) {
    if (event.button !== mouseButtons.main) {
      return;
    }

    state.mouseStatus = "up";
  }

  function onMouseMove(event: MouseEvent) {
    if (event.button !== mouseButtons.main || state.mouseStatus === "up") {
      return;
    }

    if (!state.mouseIsMoving) {
      state.mouseIsMoving = true;
    }

    const { left, top } = element.getBoundingClientRect();

    const currentMousePosition = {
      x: event.clientX - left,
      y: event.clientY - top,
    };

    const mouseMoveEvent: MouseMoveEvent = {
      type: "mousemove",
      from: state.previousMousePosition ?? currentMousePosition,
      to: currentMousePosition,
    };

    state.previousMousePosition = mouseMoveEvent.to;

    log.debug(
      "remote window events",
      log.json`Mouse move event ${mouseMoveEvent}`
    );

    listener && listener(mouseMoveEvent);
  }

  function onClick(event: MouseEvent) {
    if (
      event.button !== mouseButtons.main ||
      !event.target ||
      !element.contains(event.target as Node)
    ) {
      return;
    }

    if (state.mouseIsMoving) {
      state.mouseIsMoving = false;

      return;
    }

    const { left, top } = element.getBoundingClientRect();

    const clickEvent: ClickEvent = {
      type: "click",
      position: { x: event.clientX - left, y: event.clientY - top },
    };

    log.debug("remote window events", log.json`Click event ${clickEvent}`);

    listener && listener(clickEvent);
  }

  return { onMouseDown, onMouseMove, onMouseUp, onClick };
}

export default function remoteWindowEvents(
  element: HTMLElement,
  listener?: Listener
) {
  const state: State = {
    mouseStatus: "up",
    mouseIsMoving: false,
    previousMousePosition: null,
  };

  const { onMouseDown, onMouseMove, onMouseUp, onClick } = createEvents(
    state,
    element,
    listener
  );

  window.addEventListener("mousedown", onMouseDown);

  window.addEventListener("mouseup", onMouseUp);

  window.addEventListener("mousemove", onMouseMove);

  window.addEventListener("click", onClick);

  return {
    destroy() {
      window.removeEventListener("mousedown", onMouseDown);

      window.removeEventListener("mouseup", onMouseUp);

      window.removeEventListener("mousemove", onMouseMove);

      window.removeEventListener("click", onClick);
    },
  };
}
