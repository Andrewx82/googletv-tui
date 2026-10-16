pub mod polo {
    include!(concat!(env!("OUT_DIR"), "/polo.wire.protobuf.rs"));
}

pub mod remote {
    include!(concat!(env!("OUT_DIR"), "/remote.rs"));
}

use remote::RemoteKeyCode;

/// Translates string names into RemoteKeyCode enum variants
pub fn parse_keycode(name: &str) -> Option<RemoteKeyCode> {
    let clean = name.trim().to_uppercase();
    let normalized = if clean.starts_with("KEYCODE_") {
        clean
    } else {
        format!("KEYCODE_{}", clean)
    };

    match normalized.as_str() {
        "KEYCODE_POWER" => Some(RemoteKeyCode::KeycodePower),
        "KEYCODE_HOME" => Some(RemoteKeyCode::KeycodeHome),
        "KEYCODE_BACK" => Some(RemoteKeyCode::KeycodeBack),
        "KEYCODE_DPAD_UP" | "KEYCODE_UP" => Some(RemoteKeyCode::KeycodeDpadUp),
        "KEYCODE_DPAD_DOWN" | "KEYCODE_DOWN" => Some(RemoteKeyCode::KeycodeDpadDown),
        "KEYCODE_DPAD_LEFT" | "KEYCODE_LEFT" => Some(RemoteKeyCode::KeycodeDpadLeft),
        "KEYCODE_DPAD_RIGHT" | "KEYCODE_RIGHT" => Some(RemoteKeyCode::KeycodeDpadRight),
        "KEYCODE_DPAD_CENTER" | "KEYCODE_OK" | "KEYCODE_ENTER" | "KEYCODE_CENTER" => {
            Some(RemoteKeyCode::KeycodeDpadCenter)
        }
        "KEYCODE_VOLUME_UP" | "KEYCODE_VOLUP" => Some(RemoteKeyCode::KeycodeVolumeUp),
        "KEYCODE_VOLUME_DOWN" | "KEYCODE_VOLDOWN" => Some(RemoteKeyCode::KeycodeVolumeDown),
        "KEYCODE_VOLUME_MUTE" | "KEYCODE_MUTE" => Some(RemoteKeyCode::KeycodeVolumeMute),
        "KEYCODE_MEDIA_PLAY_PAUSE" | "KEYCODE_PLAY_PAUSE" | "KEYCODE_PLAYPAUSE" => {
            Some(RemoteKeyCode::KeycodeMediaPlayPause)
        }
        "KEYCODE_MEDIA_PLAY" | "KEYCODE_PLAY" => Some(RemoteKeyCode::KeycodeMediaPlay),
        "KEYCODE_MEDIA_PAUSE" | "KEYCODE_PAUSE" => Some(RemoteKeyCode::KeycodeMediaPause),
        "KEYCODE_MEDIA_STOP" | "KEYCODE_STOP" => Some(RemoteKeyCode::KeycodeMediaStop),
        "KEYCODE_MEDIA_NEXT" | "KEYCODE_NEXT" => Some(RemoteKeyCode::KeycodeMediaNext),
        "KEYCODE_MEDIA_PREVIOUS" | "KEYCODE_PREV" => Some(RemoteKeyCode::KeycodeMediaPrevious),
        "KEYCODE_MEDIA_REWIND" | "KEYCODE_REWIND" => Some(RemoteKeyCode::KeycodeMediaRewind),
        "KEYCODE_MEDIA_FAST_FORWARD" | "KEYCODE_FAST_FORWARD" | "KEYCODE_FFWD" => {
            Some(RemoteKeyCode::KeycodeMediaFastForward)
        }
        "KEYCODE_SEARCH" | "KEYCODE_ASSISTANT" => Some(RemoteKeyCode::KeycodeSearch),
        "KEYCODE_SETTINGS" => Some(RemoteKeyCode::KeycodeSettings),
        "KEYCODE_MENU" => Some(RemoteKeyCode::KeycodeMenu),
        "KEYCODE_TV_INPUT" | "KEYCODE_INPUT" => Some(RemoteKeyCode::KeycodeTvInput),
        "KEYCODE_CHANNEL_UP" | "KEYCODE_CHUP" => Some(RemoteKeyCode::KeycodeChannelUp),
        "KEYCODE_CHANNEL_DOWN" | "KEYCODE_CHDOWN" => Some(RemoteKeyCode::KeycodeChannelDown),
        "KEYCODE_CAPTIONS" | "KEYCODE_SUBTITLE" => Some(RemoteKeyCode::KeycodeCaptions),
        "KEYCODE_GUIDE" => Some(RemoteKeyCode::KeycodeGuide),
        "KEYCODE_INFO" => Some(RemoteKeyCode::KeycodeInfo),
        "KEYCODE_SLEEP" => Some(RemoteKeyCode::KeycodeSleep),
        "KEYCODE_WAKEUP" => Some(RemoteKeyCode::KeycodeWakeup),
        "KEYCODE_PAIRING" => Some(RemoteKeyCode::KeycodePairing),
        "KEYCODE_0" => Some(RemoteKeyCode::Keycode0),
        "KEYCODE_1" => Some(RemoteKeyCode::Keycode1),
        "KEYCODE_2" => Some(RemoteKeyCode::Keycode2),
        "KEYCODE_3" => Some(RemoteKeyCode::Keycode3),
        "KEYCODE_4" => Some(RemoteKeyCode::Keycode4),
        "KEYCODE_5" => Some(RemoteKeyCode::Keycode5),
        "KEYCODE_6" => Some(RemoteKeyCode::Keycode6),
        "KEYCODE_7" => Some(RemoteKeyCode::Keycode7),
        "KEYCODE_8" => Some(RemoteKeyCode::Keycode8),
        "KEYCODE_9" => Some(RemoteKeyCode::Keycode9),
        _ => None,
    }
}

/// Human-readable display label for a key code
pub fn keycode_display_name(code: RemoteKeyCode) -> &'static str {
    match code {
        RemoteKeyCode::KeycodePower => "Power",
        RemoteKeyCode::KeycodeHome => "Home",
        RemoteKeyCode::KeycodeBack => "Back",
        RemoteKeyCode::KeycodeDpadUp => "Up",
        RemoteKeyCode::KeycodeDpadDown => "Down",
        RemoteKeyCode::KeycodeDpadLeft => "Left",
        RemoteKeyCode::KeycodeDpadRight => "Right",
        RemoteKeyCode::KeycodeDpadCenter => "OK / Select",
        RemoteKeyCode::KeycodeVolumeUp => "Volume +",
        RemoteKeyCode::KeycodeVolumeDown => "Volume -",
        RemoteKeyCode::KeycodeVolumeMute => "Mute",
        RemoteKeyCode::KeycodeMediaPlayPause => "Play / Pause",
        RemoteKeyCode::KeycodeMediaPlay => "Play",
        RemoteKeyCode::KeycodeMediaPause => "Pause",
        RemoteKeyCode::KeycodeMediaStop => "Stop",
        RemoteKeyCode::KeycodeMediaNext => "Next",
        RemoteKeyCode::KeycodeMediaPrevious => "Previous",
        RemoteKeyCode::KeycodeMediaRewind => "Rewind",
        RemoteKeyCode::KeycodeMediaFastForward => "Fast Forward",
        RemoteKeyCode::KeycodeSearch => "Assistant / Search",
        RemoteKeyCode::KeycodeSettings => "Settings",
        RemoteKeyCode::KeycodeMenu => "Menu",
        RemoteKeyCode::KeycodeTvInput => "Input Source",
        RemoteKeyCode::KeycodeChannelUp => "Channel +",
        RemoteKeyCode::KeycodeChannelDown => "Channel -",
        RemoteKeyCode::KeycodeCaptions => "Captions",
        RemoteKeyCode::KeycodeGuide => "TV Guide",
        RemoteKeyCode::KeycodeInfo => "Info",
        RemoteKeyCode::KeycodeSleep => "Sleep",
        RemoteKeyCode::KeycodeWakeup => "Wakeup",
        RemoteKeyCode::Keycode0 => "0",
        RemoteKeyCode::Keycode1 => "1",
        RemoteKeyCode::Keycode2 => "2",
        RemoteKeyCode::Keycode3 => "3",
        RemoteKeyCode::Keycode4 => "4",
        RemoteKeyCode::Keycode5 => "5",
        RemoteKeyCode::Keycode6 => "6",
        RemoteKeyCode::Keycode7 => "7",
        RemoteKeyCode::Keycode8 => "8",
        RemoteKeyCode::Keycode9 => "9",
        _ => "Unknown",
    }
}

/// Maps an ASCII character into an Android RemoteKeyCode for hardware/on-screen keystroke injection.
pub fn char_to_keycode(c: char) -> Option<RemoteKeyCode> {
    match c {
        'a' | 'A' => Some(RemoteKeyCode::KeycodeA),
        'b' | 'B' => Some(RemoteKeyCode::KeycodeB),
        'c' | 'C' => Some(RemoteKeyCode::KeycodeC),
        'd' | 'D' => Some(RemoteKeyCode::KeycodeD),
        'e' | 'E' => Some(RemoteKeyCode::KeycodeE),
        'f' | 'F' => Some(RemoteKeyCode::KeycodeF),
        'g' | 'G' => Some(RemoteKeyCode::KeycodeG),
        'h' | 'H' => Some(RemoteKeyCode::KeycodeH),
        'i' | 'I' => Some(RemoteKeyCode::KeycodeI),
        'j' | 'J' => Some(RemoteKeyCode::KeycodeJ),
        'k' | 'K' => Some(RemoteKeyCode::KeycodeK),
        'l' | 'L' => Some(RemoteKeyCode::KeycodeL),
        'm' | 'M' => Some(RemoteKeyCode::KeycodeM),
        'n' | 'N' => Some(RemoteKeyCode::KeycodeN),
        'o' | 'O' => Some(RemoteKeyCode::KeycodeO),
        'p' | 'P' => Some(RemoteKeyCode::KeycodeP),
        'q' | 'Q' => Some(RemoteKeyCode::KeycodeQ),
        'r' | 'R' => Some(RemoteKeyCode::KeycodeR),
        's' | 'S' => Some(RemoteKeyCode::KeycodeS),
        't' | 'T' => Some(RemoteKeyCode::KeycodeT),
        'u' | 'U' => Some(RemoteKeyCode::KeycodeU),
        'v' | 'V' => Some(RemoteKeyCode::KeycodeV),
        'w' | 'W' => Some(RemoteKeyCode::KeycodeW),
        'x' | 'X' => Some(RemoteKeyCode::KeycodeX),
        'y' | 'Y' => Some(RemoteKeyCode::KeycodeY),
        'z' | 'Z' => Some(RemoteKeyCode::KeycodeZ),
        '0' => Some(RemoteKeyCode::Keycode0),
        '1' => Some(RemoteKeyCode::Keycode1),
        '2' => Some(RemoteKeyCode::Keycode2),
        '3' => Some(RemoteKeyCode::Keycode3),
        '4' => Some(RemoteKeyCode::Keycode4),
        '5' => Some(RemoteKeyCode::Keycode5),
        '6' => Some(RemoteKeyCode::Keycode6),
        '7' => Some(RemoteKeyCode::Keycode7),
        '8' => Some(RemoteKeyCode::Keycode8),
        '9' => Some(RemoteKeyCode::Keycode9),
        ' ' => Some(RemoteKeyCode::KeycodeSpace),
        ',' => Some(RemoteKeyCode::KeycodeComma),
        '.' => Some(RemoteKeyCode::KeycodePeriod),
        '-' | '_' => Some(RemoteKeyCode::KeycodeMinus),
        '=' => Some(RemoteKeyCode::KeycodeEquals),
        '+' => Some(RemoteKeyCode::KeycodePlus),
        '/' => Some(RemoteKeyCode::KeycodeSlash),
        '\\' => Some(RemoteKeyCode::KeycodeBackslash),
        ';' | ':' => Some(RemoteKeyCode::KeycodeSemicolon),
        '\'' | '"' => Some(RemoteKeyCode::KeycodeApostrophe),
        '@' => Some(RemoteKeyCode::KeycodeAt),
        '`' | '~' => Some(RemoteKeyCode::KeycodeGrave),
        '[' | '{' => Some(RemoteKeyCode::KeycodeLeftBracket),
        ']' | '}' => Some(RemoteKeyCode::KeycodeRightBracket),
        '*' => Some(RemoteKeyCode::KeycodeStar),
        '#' => Some(RemoteKeyCode::KeycodePound),
        '\n' | '\r' => Some(RemoteKeyCode::KeycodeEnter),
        '\x08' => Some(RemoteKeyCode::KeycodeDel),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_to_keycode() {
        assert_eq!(char_to_keycode('a'), Some(RemoteKeyCode::KeycodeA));
        assert_eq!(char_to_keycode('Z'), Some(RemoteKeyCode::KeycodeZ));
        assert_eq!(char_to_keycode('5'), Some(RemoteKeyCode::Keycode5));
        assert_eq!(char_to_keycode(' '), Some(RemoteKeyCode::KeycodeSpace));
        assert_eq!(char_to_keycode('.'), Some(RemoteKeyCode::KeycodePeriod));
        assert_eq!(char_to_keycode('\n'), Some(RemoteKeyCode::KeycodeEnter));
        assert_eq!(char_to_keycode('?'), None);
    }
}
