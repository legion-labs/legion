export type GamepadButtonType =
  | "South"
  | "East"
  | "North"
  | "West"
  | "C"
  | "Z"
  | "LeftTrigger"
  | "LeftTrigger2"
  | "RightTrigger"
  | "RightTrigger2"
  | "Select"
  | "Start"
  | "Mode"
  | "LeftThumb"
  | "RightThumb"
  | "DPadUp"
  | "DPadDown"
  | "DPadLeft"
  | "DPadRight";

export function fromGamepadButtonIndex(
  buttonIndex: number
): GamepadButtonType | null {
  switch (buttonIndex) {
    case 0:
      return "South";
    default:
      return null;
  }
}

export type GamepadAxisType =
  | "LeftStickX"
  | "LeftStickY"
  | "LeftZ"
  | "RightStickX"
  | "RightStickY"
  | "RightZ"
  | "DPadX"
  | "DPadY";

export function fromGamepadAxisIndex(
  axisIndex: number
): GamepadAxisType | null {
  switch (axisIndex) {
    case 0:
      return "LeftStickX";
    case 1:
      return "LeftStickY";
    default:
      return null;
  }
}
