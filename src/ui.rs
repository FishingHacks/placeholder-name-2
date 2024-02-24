use raylib::{
    color::Color,
    drawing::{RaylibDraw, RaylibDrawHandle},
    ffi::{KeyboardKey, MouseButton},
    input::key_from_i32,
    math::Rectangle,
    text::measure_text,
};

const BORDER_ACTIVE: Color = Color::new(0x04, 0x92, 0xc7, 0xff);
const COLOR_ACTIVE: Color = Color::new(0x97, 0xe8, 0xff, 0xff);

const BORDER_INACTIVE: Color = Color::BLACK;
const COLOR_INACTIVE: Color = Color::WHITE;

pub struct TextboxState {
    pub active: bool,
    pub str: String,
    pub cursor_location: usize,
    pub offset: usize,
}

impl Default for TextboxState {
    fn default() -> Self {
        Self {
            cursor_location: 6,
            offset: 0,
            active: true,
            str: String::default(),
        }
    }
}

pub fn get_key_pressed() -> Option<KeyboardKey> {
    // unsafe eater yum yum
    let key = unsafe { raylib::ffi::GetKeyPressed() };
    if key > 0 {
        return key_from_i32(key);
    }
    None
}

pub fn get_char_pressed() -> Option<u8> {
    // unsafe eater yum yum
    let key = unsafe { raylib::ffi::GetCharPressed() };
    if key > 0 {
        return key.try_into().ok();
    }
    None
}

/// If active: returns if the user clicked somewhere outside of the text box or pressed enter
///
/// If not active: returns if the user clicked somewhere inside the text box
///
/// active: `state.active`
///
/// note: this does not automatically update the state
pub fn gui_textbox(
    renderer: &mut RaylibDrawHandle,
    rect: Rectangle,
    state: &mut TextboxState,
    max_length: Option<usize>,
    tooltip: Option<&str>,
) -> bool {
    let mut return_val = false;

    if renderer.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
        let is_colliding = rect.check_collision_point_rec(renderer.get_mouse_position());
        return_val = (state.active && !is_colliding) || (!state.active && is_colliding);
    }

    if (max_length.is_none() || state.str.len() < max_length.unwrap_or(0)) && state.active {
        if let Some(char) = get_char_pressed() {
            state.str.push(char::from(char));
            state.cursor_location += 1;
        }

        if let Some(press) = get_key_pressed() {
            match press {
                KeyboardKey::KEY_LEFT if state.cursor_location > 0 => state.cursor_location -= 1,
                KeyboardKey::KEY_RIGHT if state.cursor_location < state.str.len() => {
                    state.cursor_location += 1
                }
                KeyboardKey::KEY_PAGE_UP if state.cursor_location >= 5 => {
                    state.cursor_location -= 5
                }
                KeyboardKey::KEY_PAGE_DOWN if state.cursor_location + 5 < state.str.len() => {
                    state.cursor_location += 5
                }
                KeyboardKey::KEY_UP | KeyboardKey::KEY_HOME => state.cursor_location = 0,
                KeyboardKey::KEY_DOWN | KeyboardKey::KEY_END => {
                    state.cursor_location = state.str.len()
                }
                KeyboardKey::KEY_BACKSPACE => {
                    if state.cursor_location < state.str.len() {
                        state.str.remove(state.cursor_location - 1);
                    } else if state.cursor_location <= state.str.len() {
                        state.str.pop();
                    }
                    if state.cursor_location > 0 {
                        state.cursor_location -= 1;
                    }
                }
                KeyboardKey::KEY_DELETE => {
                    if state.cursor_location < state.str.len() - 1 {
                        state.str.remove(state.cursor_location);
                    } else if state.cursor_location < state.str.len() {
                        state.str.pop();
                    }
                }
                KeyboardKey::KEY_ENTER => return_val = true,
                _ => {}
            }
        }
    }

    let x = rect.x as i32;
    let y = rect.y as i32;
    let width = rect.width as i32;
    let height = rect.height as i32;
    if state.offset >= state.cursor_location || state.offset >= state.str.len() {
        state.offset = 0;
    }
    if state.cursor_location >= state.str.len() {
        state.cursor_location = state.str.len();
    }

    let font_sz = (height - 10) / 10 * 10;
    let pad_top = ((height - font_sz) / 2) as f32;

    let mut cursor_x =
        x + measure_text(&state.str[state.offset..state.cursor_location], font_sz) + 4;
    while cursor_x + 8 >= x + width {
        state.offset += 1;
        if state.offset > state.cursor_location - 1 || state.offset >= state.str.len() - 1 {
            break;
        }
        cursor_x = x + measure_text(&state.str[state.offset..state.cursor_location], font_sz) + 4;
    }

    let (border_color, color) = if state.active {
        (BORDER_ACTIVE, COLOR_ACTIVE)
    } else {
        (BORDER_INACTIVE, COLOR_INACTIVE)
    };

    renderer.draw_rectangle(x, y, width, height, color);
    renderer.draw_rectangle_lines(x, y, width, height, border_color);

    if state.str.len() == 0 {
        if let Some(tooltip) = tooltip {
            renderer.draw_text_rec(
                renderer.get_font_default(),
                tooltip,
                Rectangle::new(
                    rect.x + 4.0,
                    rect.y + pad_top,
                    rect.width - 8.0,
                    rect.height - pad_top * 2.0,
                ),
                font_sz as f32,
                font_sz as f32 / 10.0,
                false,
                border_color.fade(0.5),
            );
        }
    } else {
        renderer.draw_text_rec(
            renderer.get_font_default(),
            &state.str[state.offset..],
            Rectangle::new(
                rect.x + 4.0,
                rect.y + pad_top,
                rect.width - 8.0,
                rect.height - pad_top * 2.0,
            ),
            font_sz as f32,
            font_sz as f32 / 10.0,
            false,
            border_color,
        );
    }

    if state.active {
        // cursor
        renderer.draw_rectangle(
            cursor_x,
            rect.y as i32 + pad_top as i32 - 1,
            2,
            font_sz + 2,
            border_color,
        );
    }

    return_val
}
