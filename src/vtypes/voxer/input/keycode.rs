use winit::keyboard::{KeyCode, PhysicalKey};

pub fn get_keycode(key: PhysicalKey) -> Option<KeyCode> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    Some(code)
}

pub fn keycode_index(key_code: KeyCode) -> Option<u8> {
    // winit KeyCode enum is not repr(u32) :D
    match key_code {
        // Letters
        KeyCode::KeyA => Some(1),
        KeyCode::KeyB => Some(2),
        KeyCode::KeyC => Some(3),
        KeyCode::KeyD => Some(4),
        KeyCode::KeyE => Some(5),
        KeyCode::KeyF => Some(6),
        KeyCode::KeyG => Some(7),
        KeyCode::KeyH => Some(8),
        KeyCode::KeyI => Some(9),
        KeyCode::KeyJ => Some(10),
        KeyCode::KeyK => Some(11),
        KeyCode::KeyL => Some(12),
        KeyCode::KeyM => Some(13),
        KeyCode::KeyN => Some(14),
        KeyCode::KeyO => Some(15),
        KeyCode::KeyP => Some(16),
        KeyCode::KeyQ => Some(17),
        KeyCode::KeyR => Some(18),
        KeyCode::KeyS => Some(19),
        KeyCode::KeyT => Some(20),
        KeyCode::KeyU => Some(21),
        KeyCode::KeyV => Some(22),
        KeyCode::KeyW => Some(23),
        KeyCode::KeyX => Some(24),
        KeyCode::KeyY => Some(25),
        KeyCode::KeyZ => Some(26),

        // Numbers
        KeyCode::Digit0 => Some(27),
        KeyCode::Digit1 => Some(28),
        KeyCode::Digit2 => Some(29),
        KeyCode::Digit3 => Some(30),
        KeyCode::Digit4 => Some(31),
        KeyCode::Digit5 => Some(32),
        KeyCode::Digit6 => Some(33),
        KeyCode::Digit7 => Some(34),
        KeyCode::Digit8 => Some(35),
        KeyCode::Digit9 => Some(36),

        // Function Keys
        KeyCode::F1 => Some(37),
        KeyCode::F2 => Some(38),
        KeyCode::F3 => Some(39),
        KeyCode::F4 => Some(40),
        KeyCode::F5 => Some(41),
        KeyCode::F6 => Some(42),
        KeyCode::F7 => Some(43),
        KeyCode::F8 => Some(44),
        KeyCode::F9 => Some(45),
        KeyCode::F10 => Some(46),
        KeyCode::F11 => Some(47),
        KeyCode::F12 => Some(48),
        KeyCode::F13 => Some(49),
        KeyCode::F14 => Some(50),
        KeyCode::F15 => Some(51),
        KeyCode::F16 => Some(52),
        KeyCode::F17 => Some(53),
        KeyCode::F18 => Some(54),
        KeyCode::F19 => Some(55),
        KeyCode::F20 => Some(56),
        KeyCode::F21 => Some(57),
        KeyCode::F22 => Some(58),
        KeyCode::F23 => Some(59),
        KeyCode::F24 => Some(60),

        // Navigation
        KeyCode::ArrowDown => Some(61),
        KeyCode::ArrowLeft => Some(62),
        KeyCode::ArrowRight => Some(63),
        KeyCode::ArrowUp => Some(64),
        KeyCode::End => Some(65),
        KeyCode::Home => Some(66),
        KeyCode::PageDown => Some(67),
        KeyCode::PageUp => Some(68),

        // Editing
        KeyCode::Backspace => Some(69),
        KeyCode::Delete => Some(70),
        KeyCode::Insert => Some(71),

        // Control
        KeyCode::AltLeft => Some(72),
        KeyCode::AltRight => Some(73),
        KeyCode::CapsLock => Some(74),
        KeyCode::ContextMenu => Some(75),
        KeyCode::ControlLeft => Some(76),
        KeyCode::ControlRight => Some(77),
        KeyCode::Enter => Some(78),
        KeyCode::SuperLeft => Some(79), // Windows/Command key
        KeyCode::SuperRight => Some(80),
        KeyCode::ShiftLeft => Some(81),
        KeyCode::ShiftRight => Some(82),
        KeyCode::Space => Some(83),
        KeyCode::Tab => Some(84),

        // System
        KeyCode::Escape => Some(85),
        KeyCode::PrintScreen => Some(86),
        KeyCode::ScrollLock => Some(87),
        KeyCode::Pause => Some(88),

        // Keypad
        KeyCode::NumLock => Some(89),
        KeyCode::Numpad0 => Some(90),
        KeyCode::Numpad1 => Some(91),
        KeyCode::Numpad2 => Some(92),
        KeyCode::Numpad3 => Some(93),
        KeyCode::Numpad4 => Some(94),
        KeyCode::Numpad5 => Some(95),
        KeyCode::Numpad6 => Some(96),
        KeyCode::Numpad7 => Some(97),
        KeyCode::Numpad8 => Some(98),
        KeyCode::Numpad9 => Some(99),
        KeyCode::NumpadAdd => Some(100),
        KeyCode::NumpadDivide => Some(101),
        KeyCode::NumpadDecimal => Some(102),
        KeyCode::NumpadComma => Some(103),
        KeyCode::NumpadEnter => Some(104),
        KeyCode::NumpadEqual => Some(105),
        KeyCode::NumpadMultiply => Some(106),
        KeyCode::NumpadSubtract => Some(107),

        // Punctuation and Symbols
        KeyCode::Backquote => Some(108),     // `
        KeyCode::Backslash => Some(109),     // \
        KeyCode::BracketLeft => Some(110),   // [
        KeyCode::BracketRight => Some(111),  // ]
        KeyCode::Comma => Some(112),         // ,
        KeyCode::Equal => Some(113),         // =
        KeyCode::IntlBackslash => Some(114), // International \
        KeyCode::IntlRo => Some(115),        // Japanese Ro key
        KeyCode::IntlYen => Some(116),       // Japanese Yen key
        KeyCode::Minus => Some(117),         // -
        KeyCode::Period => Some(118),        // .
        KeyCode::Quote => Some(119),         // '
        KeyCode::Semicolon => Some(120),     // ;
        KeyCode::Slash => Some(121),         // /

        // Asian Input Method
        KeyCode::Convert => Some(122),    // Japanese Convert
        KeyCode::KanaMode => Some(123),   // Japanese Kana
        KeyCode::Lang1 => Some(124),      // Korean Hangul
        KeyCode::Lang2 => Some(125),      // Korean Hanja
        KeyCode::Lang3 => Some(126),      // Japanese Katakana
        KeyCode::Lang4 => Some(127),      // Japanese Hiragana
        KeyCode::Lang5 => Some(128),      // Japanese Zenkaku/Hankaku
        KeyCode::NonConvert => Some(129), // Japanese Non-Convert

        // Media Keys
        KeyCode::MediaPlayPause => Some(130),
        KeyCode::MediaSelect => Some(131),
        KeyCode::MediaStop => Some(132),
        KeyCode::MediaTrackNext => Some(133),
        KeyCode::MediaTrackPrevious => Some(134),

        // Volume
        KeyCode::AudioVolumeDown => Some(135),
        KeyCode::AudioVolumeMute => Some(136),
        KeyCode::AudioVolumeUp => Some(137),

        // Browser
        KeyCode::BrowserBack => Some(138),
        KeyCode::BrowserFavorites => Some(139),
        KeyCode::BrowserForward => Some(140),
        KeyCode::BrowserHome => Some(141),
        KeyCode::BrowserRefresh => Some(142),
        KeyCode::BrowserSearch => Some(143),
        KeyCode::BrowserStop => Some(144),

        // Launch Keys
        KeyCode::LaunchApp1 => Some(145),
        KeyCode::LaunchApp2 => Some(146),
        KeyCode::LaunchMail => Some(147),

        // Power
        KeyCode::Power => Some(149),
        KeyCode::Sleep => Some(150),
        KeyCode::WakeUp => Some(151),

        // Miscellaneous
        KeyCode::Fn => Some(152),
        KeyCode::FnLock => Some(153),
        KeyCode::Help => Some(154),
        KeyCode::Undo => Some(155),
        KeyCode::Cut => Some(156),
        KeyCode::Copy => Some(157),
        KeyCode::Paste => Some(158),
        KeyCode::Find => Some(159),
        KeyCode::Open => Some(160),
        KeyCode::Props => Some(161),
        KeyCode::Select => Some(162),
        KeyCode::Again => Some(163),
        KeyCode::Abort => Some(148), // 148 was skipped earlier

        _ => None,
    }
}
