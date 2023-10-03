use ggez::conf::{WindowSetup, WindowMode};
use ggez::event;
use ggez::input::keyboard::{ScanCode, KeyboardContext};
use ggez::graphics::{self, Rect, Color, Text, DrawParam};
use ggez::{Context, GameResult};
use ggez::glam::*;
use std::time::Duration;

struct MainState {
    lines: Vec<String>,
    cursor_pos: Vec2,
    font_size: f32,
    keys_to_check: Vec<(ScanCode, char, char)>,

    window_offset: Vec2,
}

fn line_len_px(c_width: f32, n: usize) -> f32 {
    52.0 + c_width * 0.53 * n as f32 
}

//Not used, now going to just move window and have slider at bottom
/*
fn recurse_line_seperation(state_obj: &&mut MainState, curr_line: &String, line_content: RefCell<&mut Vec<String>>){
    //Modify line_content in place, no return value

    if line_len_px(state_obj.fontize, curr_line.len()) > 300.0{
        //New "line" to be added
        //Last character in this line to be moved to new "line" below
        //Seperate line at ind
        let ind: usize = f32::floor(300.0 / (line_len_px(state_obj.fontize, curr_line.len()) / curr_line.len() as f32)) as usize + 1;

        line_content.borrow_mut().push(curr_line[..ind].to_owned()); //Kinda nice
        line_content.borrow_mut().push(curr_line[ind..].to_owned());

        let new_line = &line_content.borrow().last().unwrap().clone();
        return recurse_line_seperation(state_obj, new_line, line_content)
    }
    else{
        line_content.borrow_mut().push(curr_line.clone());
    }
}
*/

impl MainState {
    fn new() -> GameResult<MainState> {
        

        //Use scancodes for british keyboard layout, other layouts can be implemented in the exact same way
        let keys_check = vec![(0x10, 'q', 'Q'), (0x11, 'w', 'W'), (0x12, 'e', 'E'), (0x13, 'r', 'R'),
                                                    (0x14, 't', 'T'), (0x15, 'y', 'Y'), (0x16, 'u', 'U'), (0x17, 'i', 'I'),
                                                    (0x18, 'o', 'O'), (0x19, 'p', 'P'), (0x1A, '[', '{'), (0x1B, ']', '}'),
                                                    (0x1E, 'a', 'A'), (0x1F, 's', 'S'), (0x20, 'd', 'D'), (0x21, 'f', 'F'),
                                                    (0x22, 'g', 'G'), (0x23, 'h', 'H'), (0x24, 'j', 'J'), (0x25, 'k', 'K'),
                                                    (0x26, 'l', 'L'), (0x27, ';', ':'), (0x28, '\'', '@'), (0x2B, '#', '~'),
                                                    (0x56, '\\', '|'), (0x2C, 'z', 'Z'), (0x2D, 'x', 'X'), (0x2E, 'c', 'C'),
                                                    (0x2F, 'v', 'V'), (0x30, 'b', 'B'), (0x31, 'n', 'N'), (0x32, 'm', 'M'),
                                                    (0x33, ',', '<'), (0x34, '.', '>'), (0x35, '/', '?'), (0x29, '`', '¬'),
                                                    (0x02, '1', '!'), (0x03, '2', '\"'), (0x04, '3', '£'), (0x05, '4', '$'),
                                                    (0x06, '5', '%'), (0x07, '6', '^'), (0x08, '7', '&'), (0x09, '8', '*'),
                                                    (0x0A, '9', '('), (0x0B, '0', ')'), (0x0C, '-', '_'), (0x0D, '=', '+'),
                                                    (0x39, ' ', ' ')
                                                    ];
        let s = MainState { lines: vec![String::new()],
             cursor_pos: Vec2::new(0.0, 0.0), font_size: 25.0,
              keys_to_check: keys_check, window_offset: Vec2::new(0.0, 0.0) };
        Ok(s)
    }

    fn ln(&mut self) -> &mut String {
        &mut self.lines[self.cursor_pos.y as usize]
    }

    fn lnr(&self) -> &String {
        &self.lines[self.cursor_pos.y as usize]
    }

    fn remove_at_cursor(&mut self) -> Option<()>{
        let temp_xpos = self.cursor_pos.x - 1.0;

        if !self.ln().is_empty() && temp_xpos > -1.0 && temp_xpos < (self.ln().len() as f32) - 1.0{            
            //Valid, remove character
            let temp_xpos = temp_xpos as usize;
            let head: &str = &self.ln().clone()[..temp_xpos];
            let tail: &str = &self.ln()[temp_xpos + 1..];

            *self.ln() = head.to_owned() + tail;
            return Some(());
        }
        else if temp_xpos != -1.0 && temp_xpos == (self.ln().len() as f32) - 1.0{
            return match self.ln().pop() {
               Some(_) => Some(()),
               _ => None
            }
        }
        None
    }

    fn get_pressed_keys(&self, kbd: &KeyboardContext) -> String {
        let mut r: String = String::new();

        for i in self.keys_to_check.iter() {

            match kbd.is_scancode_just_pressed(i.0) || (kbd.is_scancode_pressed(i.0) && kbd.is_key_repeated()){
                true if kbd.is_mod_active(ggez::input::keyboard::KeyMods::SHIFT) => r.push(i.2),
                true => r.push(i.1),
                false => ()
            };
        }
        
        r
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let keyboard = &ctx.keyboard;

        //Obtain all keys just pressed this frame, put into a string
        let key_string = self.get_pressed_keys(keyboard);
        let xpos = self.cursor_pos.x as usize;
        self.ln().insert_str(xpos, key_string.as_str());//Update line
        self.cursor_pos.x += key_string.len() as f32;

        //Handle special characters
        
        //Handle backspace
        if keyboard.is_scancode_just_pressed(0x0E) || (keyboard.is_scancode_pressed(0x0E) && keyboard.is_key_repeated()){
            
            match self.remove_at_cursor() {
                Some(_) => {
                    self.cursor_pos.x -= 1.0;
                    ()
                },
                None if self.cursor_pos.y > 0.0 => {
                    let temp_str: String = self.ln().clone(); 
                    self.lines.remove(self.cursor_pos.y as usize);
                    
                    self.cursor_pos.y -= 1.0;
                    self.cursor_pos.x = self.ln().len() as f32;
                    
                    self.ln().push_str(temp_str.as_str());
                    ()
                }
                _ => ()
            }
        }

        //Handle enter
        if keyboard.is_scancode_just_pressed(0x1C) || (keyboard.is_scancode_pressed(0x1C) && keyboard.is_key_repeated()){
            
            let mut carry: String = String::new();
            if self.ln().len() > self.cursor_pos.x as usize {
                let temp_xpos = self.cursor_pos.x as usize;
                carry = self.ln()[temp_xpos..].to_owned();
                *self.ln() = self.ln()[..temp_xpos].to_owned();
            }
            
            self.cursor_pos.y += 1.0;
            self.lines.insert(self.cursor_pos.y as usize, String::new());

            self.ln().push_str(carry.as_str());
            self.cursor_pos.x = 0.0;
        }

        //Handle arrow keys
        if keyboard.is_scancode_just_pressed(0x67) || (keyboard.is_scancode_pressed(0x67) && keyboard.is_key_repeated()){
            //Up

            //Up and down need to be careful when changing lines, if the new line has a length < the current cursor x pos then panic
            //Variable the x pos as well to match the new line
            if self.cursor_pos.y != 0.0 {
                self.cursor_pos.y -= 1.0;

                if (self.ln().len() as f32) < self.cursor_pos.x {
                    self.cursor_pos.x = self.ln().len() as f32;
                }
            };
        }
        if keyboard.is_scancode_just_pressed(0x69) || (keyboard.is_scancode_pressed(0x69) && keyboard.is_key_repeated()){
            //Left
            if self.cursor_pos.x != 0.0 {self.cursor_pos.x -= 1.0};
        }
        if keyboard.is_scancode_just_pressed(0x6C) || (keyboard.is_scancode_pressed(0x6C) && keyboard.is_key_repeated()){
            //Down
            //Match up key when new line length < .. etc.
            if self.cursor_pos.y < self.lines.len() as f32 - 1.0 {
                self.cursor_pos.y += 1 as f32;

                if (self.ln().len() as f32) < self.cursor_pos.x {
                    self.cursor_pos.x = self.ln().len() as f32;
                }
            };
        }
        if keyboard.is_scancode_just_pressed(0x6A) || (keyboard.is_scancode_pressed(0x6A) && keyboard.is_key_repeated()){
            //Right
            if self.cursor_pos.x != self.ln().len() as f32 {self.cursor_pos.x += 1.0};
        }

        //Handle tabs
        if keyboard.is_scancode_just_pressed(0x0F) || (keyboard.is_scancode_pressed(0x0F) && keyboard.is_key_repeated()){
            //Tab Pressed - just add 4 spaces to string
            let t_xpos = self.cursor_pos.x as usize;
            self.ln().insert_str(t_xpos, "    ");
            self.cursor_pos.x += 4.0;
        }
        ggez::timer::sleep(Duration::from_secs((1.0 / ctx.time.fps() - 1.0 / 60.0) as u64)); // Cap to 60fps
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            Color::from_rgb(40, 40, 40),
        );

        //See if cursor is outside of screen, and if so then increment offsets for text and cursor.
        //Every drawable inside of the text area will be given this offset
        if line_len_px(self.font_size, self.cursor_pos.x as usize ) > ctx.gfx.window().inner_size().width as f32 + self.window_offset.x {
            //Overflow
            self.window_offset.x += self.font_size * 0.53; //Conversion constant
        }
        else if line_len_px(self.font_size, self.cursor_pos.x as usize ) < self.window_offset.x + 50.0{
            self.window_offset.x -= self.font_size * 0.53; //Conversion constant
        }

        //Draw off limits bar (Where the line number reside)
        //Has to be before num lines
        let off_limits_rect = Rect::new(0.0, 0.0, 50.0, 10000000000.0);
        let off_limits_rect_mash = ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill()
                                                                                , off_limits_rect, Color::from_rgb(30, 30, 30))?;
        canvas.draw(&off_limits_rect_mash, DrawParam::default());

        //Draw text-editor interface(lines)
        let mut overflow_constant: f32 = 0.0;
        for (i, line) in self.lines.iter().enumerate() {
            //Loop through every line and construct text
          //  let mut line_content: Vec<String> = Vec::new();

            
          //  recurse_line_seperation(&self, line, RefCell::new(&mut line_content)); //Yikes
            

            let line_num = Text::new(ggez::graphics::TextFragment {
                text: (i + 1).to_string(),
                color: Some(Color::from_rgb(140, 140, 140)),
                font: Some("LiberationMono-Regular".into()),
                scale: Some(ggez::graphics::PxScale::from(self.font_size)),

            });

            /*
            //Convert line_content from Vec<String> to Vec<Text>
            let line_content: Vec<Text> = line_content.iter()
                .map(|lin| Text::new(ggez::graphics::TextFragment {
                    text: lin.clone(),
                    color: Some(Color::from_rgb(255, 255, 255)),
                    font: Some("LiberationMono-Regular".into()),
                    scale: Some(ggez::graphics::PxScale::from(self.fontize)),
                }))
                .collect();
            */

            /*
            for (extra_line_ind, lc) in line_content.iter().enumerate() {
                canvas.draw(lc, Vec2::new(50.0, self.fontize * (i + extra_line_ind) as f32));
            }
            */
            let line_content = Text::new(ggez::graphics::TextFragment {
                    text: line.clone(),
                    color: Some(Color::from_rgb(255, 255, 255)),
                    font: Some("LiberationMono-Regular".into()),
                    scale: Some(ggez::graphics::PxScale::from(self.font_size)),
            });
            canvas.draw(&line_content, Vec2::new(50.0, self.font_size * i as f32) - self.window_offset);
            canvas.draw(&line_num, Vec2::new(0.0, self.font_size * i as f32 + overflow_constant));
        }

        //Draw cursor
     //   let rect = Rect::new(transIndexWidth(text_vec[self.cursor_pos.y as usize].clone(), self.cursor_pos.x as usize), 
      //  15.0 * self.cursor_pos.y, 2.0, 15.0);
       // let seperator = Rect::new(50.0, 0.0, 2.0, 1080.0);
     //   let seperator_mesh = ggez::graphics::Mesh::new_rectangle(ctx,
             //ggez::graphics::DrawMode::fill(),seperator, Color::from_rgb(180, 180, 180))?;             



        let window = ctx.gfx.window();
        
        //Cursor
        let mut rect = Rect::new(line_len_px(self.font_size, self.cursor_pos.x as usize), //fix
             self.font_size * self.cursor_pos.y + overflow_constant, 2.0, self.font_size);
        rect.x -= self.window_offset.x;
        rect.y -= self.window_offset.y;

        let rect_mesh = ggez::graphics::Mesh::new_rectangle(ctx, ggez::graphics::DrawMode::fill(),
                                                                      rect, Color::from_rgb(255, 255, 255))?;

        canvas.draw(&rect_mesh, DrawParam::default());
        canvas.finish(ctx)?;
        Ok(())
    }
}

pub fn main() -> GameResult {
    let mut cb = ggez::ContextBuilder::new("Vio", "Robert West");

    cb = cb.window_setup(WindowSetup {
        title: "Vio, Strange Vi".to_string(),
        samples: ggez::conf::NumSamples::One,
        vsync: true,
        icon: "".to_string(),
        srgb: true
    });
    cb = cb.window_mode(WindowMode {
        width: 800.0,
        height: 600.0,
        maximized: false,
        fullscreen_type: ggez::conf::FullscreenType::Windowed,
        borderless: false,
        min_width: 200.0,
        max_width: 1920.0,
        min_height: 200.0,
        max_height: 1080.0,
        resizable: true,
        visible: true,
        resize_on_scale_factor_change: false,
        transparent: false,
        logical_size: None,
    });

    let (ctx, event_loop) = cb.build()?;
    let state = MainState::new()?;
    event::run(ctx, event_loop, state)
}