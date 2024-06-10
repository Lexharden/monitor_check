use chrono::{DateTime, Local};
use device_query::{DeviceQuery, DeviceState, Keycode};
use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct InputActivity {
    #[serde(serialize_with = "serialize_keycode_vec")]
    pub keys_pressed: Vec<Keycode>,
    pub mouse_position: (i32, i32),
    #[serde(with = "my_date_format")]
    pub last_activity: DateTime<Local>,
    pub typed_text: String,
}

mod my_date_format {
    use chrono::{DateTime, Local, SecondsFormat};
    use serde::{self, Serializer, Deserializer, Deserialize};

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = date.to_rfc3339_opts(SecondsFormat::Secs, true);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Local))
    }
}

// Implementación personalizada para serializar `Vec<Keycode>`
fn serialize_keycode_vec<S>(vec: &Vec<Keycode>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
{
    let strings: Vec<String> = vec.iter().map(|k| format!("{:?}", k)).collect();
    serializer.serialize_str(&strings.join(","))
}

pub fn get_input_activity(last_activity: &mut DateTime<Local>) -> InputActivity {
    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    let mouse: (i32, i32) = device_state.get_mouse().coords;
    let mut typed_text = String::new();

    let now = Local::now();
    if !keys.is_empty() || mouse != (0, 0) {
        *last_activity = now;
    }

    for key in &keys {
        match key {
            Keycode::Space => typed_text.push(' '),
            Keycode::Backspace => {
                typed_text.pop();
            }
            Keycode::A => typed_text.push('a'),
            Keycode::B => typed_text.push('b'),
            Keycode::C => typed_text.push('c'),
            Keycode::D => typed_text.push('d'),
            Keycode::E => typed_text.push('e'),
            Keycode::F => typed_text.push('f'),
            Keycode::G => typed_text.push('g'),
            Keycode::H => typed_text.push('h'),
            Keycode::I => typed_text.push('i'),
            Keycode::J => typed_text.push('j'),
            Keycode::K => typed_text.push('k'),
            Keycode::L => typed_text.push('l'),
            Keycode::M => typed_text.push('m'),
            Keycode::N => typed_text.push('n'),
            Keycode::O => typed_text.push('o'),
            Keycode::P => typed_text.push('p'),
            Keycode::Q => typed_text.push('q'),
            Keycode::R => typed_text.push('r'),
            Keycode::S => typed_text.push('s'),
            Keycode::T => typed_text.push('t'),
            Keycode::U => typed_text.push('u'),
            Keycode::V => typed_text.push('v'),
            Keycode::W => typed_text.push('w'),
            Keycode::X => typed_text.push('x'),
            Keycode::Y => typed_text.push('y'),
            Keycode::Z => typed_text.push('z'),
            Keycode::Key0 => typed_text.push('0'),
            Keycode::Key1 => typed_text.push('1'),
            Keycode::Key2 => typed_text.push('2'),
            Keycode::Key3 => typed_text.push('3'),
            Keycode::Key4 => typed_text.push('4'),
            Keycode::Key5 => typed_text.push('5'),
            Keycode::Key6 => typed_text.push('6'),
            Keycode::Key7 => typed_text.push('7'),
            Keycode::Key8 => typed_text.push('8'),
            Keycode::Key9 => typed_text.push('9'),
            _ => {}
        }
    }

    // Mensaje de depuración para verificar el texto escrito
    // println!("Texto escrito: {}", typed_text);

    InputActivity {
        keys_pressed: keys,
        mouse_position: mouse,
        last_activity: *last_activity,
        typed_text,
    }
}
