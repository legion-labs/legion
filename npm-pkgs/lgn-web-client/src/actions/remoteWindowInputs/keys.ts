export type KeyCode =
  // The '1' key over the letters.
  | "Key1"
  // The '2' key over the letters.
  | "Key2"
  // The '3' key over the letters.
  | "Key3"
  // The '4' key over the letters.
  | "Key4"
  // The '5' key over the letters.
  | "Key5"
  // The '6' key over the letters.
  | "Key6"
  // The '7' key over the letters.
  | "Key7"
  // The '8' key over the letters.
  | "Key8"
  // The '9' key over the letters.
  | "Key9"
  // The '0' key over the 'O' and 'P' keys.
  | "Key0"
  | "A"
  | "B"
  | "C"
  | "D"
  | "E"
  | "F"
  | "G"
  | "H"
  | "I"
  | "J"
  | "K"
  | "L"
  | "M"
  | "N"
  | "O"
  | "P"
  | "Q"
  | "R"
  | "S"
  | "T"
  | "U"
  | "V"
  | "W"
  | "X"
  | "Y"
  | "Z"

  // The Escape "key"| next to F1.
  | "Escape"
  | "F1"
  | "F2"
  | "F3"
  | "F4"
  | "F5"
  | "F6"
  | "F7"
  | "F8"
  | "F9"
  | "F10"
  | "F11"
  | "F12"
  | "F13"
  | "F14"
  | "F15"
  | "F16"
  | "F17"
  | "F18"
  | "F19"
  | "F20"
  | "F21"
  | "F22"
  | "F23"
  | "F24"

  // Print Screen/SysRq.
  | "Snapshot"
  // Scroll Lock.
  | "Scroll"
  // Pause/Break "key"| next to Scroll lock.
  | "Pause"

  // `"Insert`"| next to Backspace.
  | "Insert"
  | "Home"
  | "Delete"
  | "End"
  | "PageDown"
  | "PageUp"
  | "Left"
  | "Up"
  | "Right"
  | "Down"

  // The Backspace "key"| right over Enter.
  | "Back"
  // The Enter key.
  | "Return"
  // The space bar.
  | "Space"

  // The "Compose" key on Linux.
  | "Compose"
  | "Caret"
  | "Numlock"
  | "Numpad0"
  | "Numpad1"
  | "Numpad2"
  | "Numpad3"
  | "Numpad4"
  | "Numpad5"
  | "Numpad6"
  | "Numpad7"
  | "Numpad8"
  | "Numpad9"
  | "AbntC1"
  | "AbntC2"
  | "NumpadAdd"
  | "Apostrophe"
  | "Apps"
  | "Asterisk"
  | "Plus"
  | "At"
  | "Ax"
  | "Backslash"
  | "Calculator"
  | "Capital"
  | "Colon"
  | "Comma"
  | "Convert"
  | "NumpadDecimal"
  | "NumpadDivide"
  | "Equals"
  | "Grave"
  | "Kana"
  | "Kanji"

  // The left alt key. Maps to left option on Mac.
  | "LAlt"
  | "LBracket"
  | "LControl"
  | "LShift"
  // The left Windows key. Maps to left Command on Mac.
  | "LWin"
  | "Mail"
  | "MediaSelect"
  | "MediaStop"
  | "Minus"
  | "NumpadMultiply"
  | "Mute"
  | "MyComputer"
  | "NavigateForward" // also called "Prior"
  | "NavigateBackward" // also called "Next"
  | "NextTrack"
  | "NoConvert"
  | "NumpadComma"
  | "NumpadEnter"
  | "NumpadEquals"
  | "Oem102"
  | "Period"
  | "PlayPause"
  | "Power"
  | "PrevTrack"
  // The right alt key. Maps to right option on Mac.
  | "RAlt"
  | "RBracket"
  | "RControl"
  | "RShift"
  // The right Windows key. Maps to right Command on Mac.
  | "RWin"
  | "Semicolon"
  | "Slash"
  | "Sleep"
  | "Stop"
  | "NumpadSubtract"
  | "Sysrq"
  | "Tab"
  | "Underline"
  | "Unlabeled"
  | "VolumeDown"
  | "VolumeUp"
  | "Wake"
  | "WebBack"
  | "WebFavorites"
  | "WebForward"
  | "WebHome"
  | "WebRefresh"
  | "WebSearch"
  | "WebStop"
  | "Yen"
  | "Copy"
  | "Paste"
  | "Cut";

export function fromBrowserKey(key: string, location: number): KeyCode | null {
  switch (key) {
    case "0":
      return "Key0";
    case "1":
      return "Key1";
    case "2":
      return "Key2";
    case "3":
      return "Key3";
    case "4":
      return "Key4";
    case "5":
      return "Key5";
    case "6":
      return "Key6";
    case "7":
      return "Key7";
    case "8":
      return "Key8";
    case "9":
      return "Key9";

    case "a":
    case "A":
      return "A";
    case "b":
    case "B":
      return "B";
    case "c":
    case "C":
      return "C";
    case "d":
    case "D":
      return "D";
    case "e":
    case "E":
      return "E";
    case "f":
    case "F":
      return "F";
    case "g":
    case "G":
      return "G";
    case "h":
    case "H":
      return "H";
    case "i":
    case "I":
      return "I";
    case "j":
    case "J":
      return "J";
    case "k":
    case "K":
      return "K";
    case "l":
    case "L":
      return "L";
    case "m":
    case "M":
      return "M";
    case "n":
    case "N":
      return "N";
    case "o":
    case "O":
      return "O";
    case "p":
    case "P":
      return "P";
    case "q":
    case "Q":
      return "Q";
    case "r":
    case "R":
      return "R";
    case "s":
    case "S":
      return "S";
    case "t":
    case "T":
      return "T";
    case "u":
    case "U":
      return "U";
    case "v":
    case "V":
      return "V";
    case "w":
    case "W":
      return "W";
    case "x":
    case "X":
      return "X";
    case "y":
    case "Y":
      return "Y";
    case "z":
    case "Z":
      return "Z";

    case " ":
      return "Space";
    case "'":
      return "Apostrophe";
    case "*":
      return "Asterisk";
    case "+":
      return "Plus";
    case "@":
      return "At";
    case "\\":
      return "Backslash";
    case ":":
      return "Colon";
    case ",":
      return "Comma";
    case "=":
      return "Equals";
    case "`":
      return "Grave";
    case "-":
      return "Minus";
    case ".":
      return "Period";
    case "(":
      return "LBracket";
    case ")":
      return "RBracket";
    case ";":
      return "Semicolon";
    case "/":
      return "Slash";
    case "_":
      return "Underline";

    case "Alt":
      switch (location) {
        case KeyboardEvent.DOM_KEY_LOCATION_RIGHT:
          return "RAlt";
        case KeyboardEvent.DOM_KEY_LOCATION_LEFT:
          return "LAlt";
        default:
          return null;
      }

    case "Control":
      switch (location) {
        case KeyboardEvent.DOM_KEY_LOCATION_RIGHT:
          return "RControl";
        case KeyboardEvent.DOM_KEY_LOCATION_LEFT:
          return "LControl";
        default:
          return null;
      }

    case "Shift":
      switch (location) {
        case KeyboardEvent.DOM_KEY_LOCATION_RIGHT:
          return "RShift";
        case KeyboardEvent.DOM_KEY_LOCATION_LEFT:
          return "LShift";
        default:
          return null;
      }

    default:
      return null;
  }
}
