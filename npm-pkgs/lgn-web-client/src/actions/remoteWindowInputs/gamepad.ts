// Reference: https://w3c.github.io/gamepad/#remapping

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
    case 1:
      return "East";
    case 2:
      return "West";
    case 3:
      return "North";
    case 4:
      return "LeftTrigger2";
    case 5:
      return "RightTrigger2";
    case 6:
      return "LeftTrigger";
    case 7:
      return "RightTrigger";
    case 8:
      return "Select";
    case 9:
      return "Start";
    case 10:
      return "LeftThumb";
    case 11:
      return "RightThumb";
    case 12:
      return "DPadUp";
    case 13:
      return "DPadDown";
    case 14:
      return "DPadLeft";
    case 15:
      return "DPadRight";
    case 16:
      return "Mode";
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

export type GamepadAxisDescription = {
  axisType: GamepadAxisType;
  inverted: boolean;
};

export function fromGamepadAxisIndex(
  axisIndex: number
): GamepadAxisDescription | null {
  switch (axisIndex) {
    case 0:
      return { axisType: "LeftStickX", inverted: false };
    case 1:
      return { axisType: "LeftStickY", inverted: true };
    case 2:
      return { axisType: "RightStickX", inverted: false };
    case 3:
      return { axisType: "RightStickY", inverted: true };
    default:
      return null;
  }
}
