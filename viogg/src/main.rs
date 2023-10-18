use ggez::conf::{WindowSetup, WindowMode};
use ggez::event::{self, EventHandler};
use ggez::input::{keyboard::{ScanCode, KeyboardContext}, mouse::MouseButton};
use ggez::graphics::{self, Rect, Color, Text, DrawParam, Mesh, DrawMode};
use ggez::mint::Point2;
use ggez::{Context, GameResult};
use ggez::glam::*;
use std::fs::DirEntry;
use std::path;
use std::time::Duration;
use std::{env, fs, io, path::{Path, PathBuf}};

//Terrible code up ahead

struct DirectoryNode {
    entry: DirEntry,
    children: Vec<DirectoryNode>,
    opened: bool,
    metadata: fs::Metadata,

    click_rect: Rect,
}

impl DirectoryNode {
    pub fn new(ent: DirEntry) -> Self{


        let p = ent.path();
        Self {entry: ent, children: Vec::new(), opened: false, metadata: fs::metadata(p).unwrap(), click_rect: Rect::new(0.0, 0.0, 0.0, 0.0)}
    }

    fn open(&mut self) -> GameResult<bool>{
        self.opened = true;
        //populate children from whats after entry
        let mut result: bool = false;
        self.children = match initialise_directory(self.entry.path()) {
            Ok(res) => res,
            _ => {
                //File is opened
                result = true;
                Vec::new()
            }
        };

        Ok(result)
    }

    fn close(&mut self) {
        self.opened = false;
        self.children = Vec::new();
    }

    fn draw_canvas(&mut self, canvas: &mut graphics::Canvas, font_size: f32, index_pair: &mut (u32, u32), offset: Vec2) -> GameResult<()>{
        
        let mut path_string = self.entry.path().as_path().to_str().unwrap().to_owned();
        path_string = path_string.split(&['/','\'']).last().unwrap().to_owned();
        if self.metadata.is_dir() {
            path_string.insert_str(0, "./");
        }

        let len_path_string = path_string.len();

        let draw_text = Text::new(ggez::graphics::TextFragment {
                text: path_string,
                color: Some(Color::from_rgb(190, 190, 230)),
                font: Some("LiberationMono-Regular".into()),
                scale: Some(ggez::graphics::PxScale::from(font_size)),
        });


        self.click_rect.x = 10.0 + index_pair.0 as f32 * 15.0 + offset.x;
        self.click_rect.y = 40.0 + index_pair.1 as f32 * font_size + offset.y;
        self.click_rect.w = font_size * 0.53 * len_path_string as f32;
        self.click_rect.h = font_size;

        canvas.draw(&draw_text, Vec2::new(self.click_rect.x, self.click_rect.y));
        index_pair.1 += 1;

        let past_i0 = index_pair.0;

        if !self.children.is_empty() {
            index_pair.0 += 1;
        }

        for child in self.children.iter_mut() {
            child.draw_canvas(canvas, font_size, index_pair, offset)?;
        }
        index_pair.0 = past_i0;   

        Ok(())
    }
}

fn initialise_directory(path_buf: PathBuf) -> GameResult<Vec<DirectoryNode>>{
    let mut ret: Vec<DirectoryNode> = Vec::new();

    let reader = fs::read_dir(path_buf)?;
    for dir_entry in reader {
        let dir_entry = dir_entry?;

        ret.push(DirectoryNode::new(dir_entry));
    }


    Ok(ret)
}

fn recurse_click_check_direc (directory: &mut [DirectoryNode], mouse_pos: Point2<f32>) -> GameResult<Option<String>> {

    let mut file_type_result: Option<String> = None;

    for direc in directory.iter_mut() {
        if direc.click_rect.contains(mouse_pos) {
            
        //Clicking the rect
            match direc.opened {
                false => {
                    if direc.open()? {
                        //If true then it is a file not a folder
                        file_type_result = Some(direc.entry.path().to_str().unwrap().to_owned());
                    }
                }
                _ => direc.close(),
            }
        }
        if !direc.children.is_empty(){
            //Recurse
            if let Some(res) = recurse_click_check_direc(&mut direc.children, mouse_pos)? {
                file_type_result = Some(res);
            }
        }
    }
    Ok(file_type_result)
}

struct TextWindow {
    lines: Vec<String>,
    cursor_pos: Vec2,
    longest_length: f32,
}

impl TextWindow {
    pub fn new(lines_p: Vec<String>) -> Self{
        Self {lines: lines_p, cursor_pos: Vec2::new(0.0, 0.0), longest_length: 0.0 }
    }

    fn load_from_file(&mut self, path: String) -> GameResult<()>{
        let full_read: String = String::from_utf8(fs::read(path)?.to_vec()).unwrap();
        self.lines = full_read.split('\n').map(|s| s.to_owned()).collect();
        println!("number of lines: {}", self.lines.len() + 1);
        Ok(())
    }


}


struct MainState {

    windows: Vec<TextWindow>,
    curr_window: usize,

    font_size: f32,
    keys_to_check: Vec<(ScanCode, char, char)>,

    window_offset: Vec2,
    padding: Vec2,

    //Slider
    //slider_rect: Rect,
   // slider_clicked: bool,


    //Text area rect
    text_area: Rect,

    //Directory
    top_level_directory: Vec<DirectoryNode>
}

fn line_len_px(c_width: f32, n: usize) -> f32 {
    52.0 + c_width * 0.53 * n as f32 
}


impl MainState {
    fn new(ctx: &Context) -> GameResult<MainState> {
        

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


        let mut text_window = TextWindow::new(Vec::new());
        let cwd = env::current_dir().unwrap().as_path().to_str().unwrap().to_owned();

        text_window.load_from_file(cwd + "/src/main.rs")?;

        let s = MainState { 
            windows: vec!(text_window),
            curr_window: 0,
            font_size: 25.0, //Check this value
            keys_to_check: keys_check, window_offset: Vec2::new(0.0, 0.0), padding: Vec2::new(20.0, 20.0),
            //slider_rect: Rect::new(0.0, 0.0, 0.0, 10.0), slider_clicked: false,
            text_area: Rect::new(250.0, 40.0, ctx.gfx.window().inner_size().width as f32 - (250.0),
             ctx.gfx.window().inner_size().height as f32 - 40.0),
            top_level_directory: initialise_directory(env::current_dir()?)?,
         };
        Ok(s)
    }

/* slider not used
    fn update_page_slider_rect(&mut self, ctx: &Context) -> &Rect{

        let w_size = ctx.gfx.window().inner_size(); //Window size
        
        //Find longest line
        self.longest_length = 0.0;

        for line in self.win().lines.iter() {
            let new_len = line_len_px(self.font_size, line.len());
            self.longest_length = if new_len > self.longest_length {new_len} else {self.longest_length}
        }

        let text_win_ratio = w_size.width as f32 / self.longest_length;

        self.slider_rect.x = 50.0 + self.window_offset.x * text_win_ratio;
        self.slider_rect.y = w_size.height as f32 - 10.0;
        self.slider_rect.w = text_win_ratio * (w_size.width as f32 - 50.0); 

        &self.slider_rect
    }

    fn move_page_slider(&mut self, mouse_delta: Point2<f32>, ctx: &Context) {
        let win_width = ctx.gfx.window().inner_size().width as f32;
        
        let interim = self.slider_rect.x + mouse_delta.x;
        println!("interim: {}", interim);
        println!("original {} delta {}", self.slider_rect.x, mouse_delta.x);

        if interim >= 50.0 && interim + self.slider_rect.w <= win_width { //Check range
            self.slider_rect.x = interim;
            println!("Moved.");
            //Update window offset
            self.window_offset.x = (self.slider_rect.x - 50.0) * (self.longest_length / win_width);
        }
    }  
    */
    fn explicit_update_window_offset(&mut self, ctx: &Context) {

        //See if cursor is outside of screen, and if so then increment offsets for text and cursor.
        //Every drawable inside of the text area will be given this offset
        
        //X offset

        if line_len_px(self.font_size, self.win().cursor_pos.x as usize ) > self.text_area.w + self.window_offset.x {
            //Overflow
            self.window_offset.x += self.font_size * 0.53; //Conversion constant
        }
        else if line_len_px(self.font_size, self.win().cursor_pos.x as usize ) < self.window_offset.x + 50.0{
            self.window_offset.x -= self.font_size * 0.53; //Conversion constant
        }
        
        //Y offset
        if self.font_size * (self.win().cursor_pos.y + 3.0) > self.text_area.h + self.window_offset.y {
            self.window_offset.y += self.font_size; //No conversion constant 
        }
        else if self.font_size * (self.win().cursor_pos.y) < self.window_offset.y { 
            self.window_offset.y -= self.font_size;
        }
    } 


    

    fn ln(&mut self) -> &mut String {
        let y_pos = self.win().cursor_pos.y;
        &mut self.win_mut().lines[y_pos as usize]
    }

    fn lnr(&self) -> &String {
        &self.win().lines[self.win().cursor_pos.y as usize]
    }

    fn win_mut(&mut self) -> &mut TextWindow {
        &mut self.windows[self.curr_window]
    }

    fn win(&self) -> &TextWindow {
        &self.windows[self.curr_window]
    }

    fn bounded_lines(&self, ctx: &Context) -> &[String] {
        //Calculate line boundaries here
        //Line boundaries designate which lines are within the text_area according to the window offset

        let lower = (self.window_offset.y / self.font_size).floor() as usize;
        let upper = ((self.window_offset.y + ctx.gfx.window().inner_size().height as f32) / self.font_size).floor() as usize;

        &self.win().lines[lower..=upper]
    }

    fn remove_at_cursor(&mut self) -> Option<char>{
        let temp_xpos = self.win().cursor_pos.x - 1.0;

        if !self.ln().is_empty() && temp_xpos > -1.0 && temp_xpos < (self.ln().len() as f32) - 1.0{            
            //Valid, remove character
            let temp_xpos = temp_xpos as usize;
            let c = self.lnr()[temp_xpos..=temp_xpos].chars().next(); //Get removed char

            let head: &str = &self.ln().clone()[..temp_xpos];
            let tail: &str = &self.ln()[temp_xpos + 1..];

            *self.ln() = head.to_owned() + tail;
            return c;
        }
        else if temp_xpos != -1.0 && temp_xpos == (self.ln().len() as f32) - 1.0{
            return self.ln().pop();
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


        let xpos = self.win().cursor_pos.x as usize;
        self.ln().insert_str(xpos, key_string.as_str());//Update line
        self.win_mut().cursor_pos.x += key_string.len() as f32;
        
        if !key_string.is_empty() {
            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }

        //Handle special characters
        
        //Handle backspace
        if keyboard.is_scancode_just_pressed(0x0E) || (keyboard.is_scancode_pressed(0x0E) && keyboard.is_key_repeated()){
            
            match self.remove_at_cursor() {
                Some(c) => {
                    self.win_mut().cursor_pos.x -= 1.0;

                    if self.win().cursor_pos.x == 0.0 {self.window_offset.x = 0.0}
                    else if c == ' ' && self.lnr().len() >= 3
                                     && &self.lnr()[self.win().cursor_pos.x as usize - 3..self.win().cursor_pos.x as usize] == "   " {
                        //This removes tabs all at once safely.(I swear its safe).
                        self.win_mut().cursor_pos.x -= 3.0;
                    }
                },
                None if self.win().cursor_pos.y > 0.0 => {
                    let temp_str: String = self.ln().clone(); 
                    let y_pos = self.win().cursor_pos.y;
                    self.win_mut().lines.remove(y_pos as usize);
                    
                    
                    //Calculate new cursor position
                    self.win_mut().cursor_pos.y -= 1.0;
                    self.win_mut().cursor_pos.x = self.ln().len() as f32;

                    //Calculate new window offset
                    let new_line_width = line_len_px(self.font_size, self.ln().len());
                    self.window_offset.x = if new_line_width > self.text_area.w {new_line_width - self.text_area.w + self.font_size} else {self.window_offset.x};
                    
                    self.ln().push_str(temp_str.as_str());
                }
                _ => ()
            }
           // self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }

        //Handle enter
        if keyboard.is_scancode_just_pressed(0x1C) || (keyboard.is_scancode_pressed(0x1C) && keyboard.is_key_repeated()){
            
            let mut carry: String = String::new();
            if self.ln().len() > self.win().cursor_pos.x as usize {
                let temp_xpos = self.win().cursor_pos.x as usize;
                carry = self.ln()[temp_xpos..].to_owned();
                *self.ln() = self.ln()[..temp_xpos].to_owned();
            }
            
            self.win_mut().cursor_pos.y += 1.0;
            let y_pos = self.win().cursor_pos.y;
            self.win_mut().lines.insert(y_pos as usize, String::new());

            self.ln().push_str(carry.as_str());
            self.win_mut().cursor_pos.x = 0.0;
            
            self.window_offset.x = 0.0;

            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }

        //Handle arrow keys
        if keyboard.is_scancode_just_pressed(0x67) || (keyboard.is_scancode_pressed(0x67) && keyboard.is_key_repeated()){
            //Up

            //Up and down need to be careful when changing lines, if the new line has a length < the current cursor x pos then panic
            //Variable the x pos as well to match the new line
            if self.win().cursor_pos.y != 0.0 {
                self.win_mut().cursor_pos.y -= 1.0;

                if (self.ln().len() as f32) < self.win().cursor_pos.x {
                    self.win_mut().cursor_pos.x = self.ln().len() as f32;
                }
            };

            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }
        if keyboard.is_scancode_just_pressed(0x69) || (keyboard.is_scancode_pressed(0x69) && keyboard.is_key_repeated()){
            //Left
            if self.win().cursor_pos.x != 0.0 {self.win_mut().cursor_pos.x -= 1.0};
            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }
        if keyboard.is_scancode_just_pressed(0x6C) || (keyboard.is_scancode_pressed(0x6C) && keyboard.is_key_repeated()){
            //Down
            //Match up key when new line length < .. etc.
            if self.win().cursor_pos.y < self.win().lines.len() as f32 - 1.0 {
                self.win_mut().cursor_pos.y += 1.0;

                if (self.ln().len() as f32) < self.win().cursor_pos.x {
                    self.win_mut().cursor_pos.x = self.ln().len() as f32;
                }
            };
            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }
        if keyboard.is_scancode_just_pressed(0x6A) || (keyboard.is_scancode_pressed(0x6A) && keyboard.is_key_repeated()){
            //Right
            if self.win().cursor_pos.x != self.ln().len() as f32 {self.win_mut().cursor_pos.x += 1.0};
            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }

        //Handle tabs
        if keyboard.is_scancode_just_pressed(0x0F) || (keyboard.is_scancode_pressed(0x0F) && keyboard.is_key_repeated()){
            //Tab Pressed - just add 4 spaces to string
            let t_xpos = self.win().cursor_pos.x as usize;
            self.ln().insert_str(t_xpos, "    "); //Have to use spaces since tabs are a unicode character
            self.win_mut().cursor_pos.x += 4.0;
            //self.update_page_slider_rect(ctx);
            self.explicit_update_window_offset(ctx);
        }
        

        //Handle mouse input
        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            //Mouse button pressed
            let mouse_pos = ctx.mouse.position();
            
            let click_direc_result = recurse_click_check_direc(&mut self.top_level_directory, mouse_pos)?;
            if let Some(res) = click_direc_result {
                //There was a file pressed, reload the window
                println!("here");
                self.win_mut().load_from_file(res)?;
            }

            /* - Not used
            let mouse_pos = ctx.mouse.position();
            if self.slider_rect.contains(mouse_pos) {
                //Clicking slider
                self.slider_clicked = true;
            }
            */
        }
        /*
        else if ctx.mouse.button_pressed(MouseButton::Left) {
            
            //Mouse button continuing to be pressed
            //Check page slider - not 
            //let mouse_pos = ctx.mouse.position();

            /*
            if self.slider_clicked {
                self.move_page_slider(ctx.mouse.delta(), ctx);
                println!("Slider clicked");
            }
            */
        }

        else if ctx.mouse.button_just_released(MouseButton::Left) {
            //Mouse button released
           // self.slider_clicked = false;
        }
        */
        
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(
            ctx,
            Color::from_rgb(25, 25, 40),
        );

        

        //Draw text-editor interface(lines)
        for (i, line) in self.bounded_lines(ctx).iter().enumerate() {
            //Loop through every line and construct text
            let line_content = Text::new(ggez::graphics::TextFragment {
                    text: line.clone(),
                    color: Some(Color::from_rgb(235, 235, 235)),
                    font: Some("LiberationMono-Regular".into()),
                    scale: Some(ggez::graphics::PxScale::from(self.font_size)),
            });
            canvas.draw(&line_content, Vec2::new(self.text_area.x + 50.0 - self.window_offset.x + self.padding.x,
                        self.text_area.y + self.font_size * i as f32 + self.padding.y));
        }

        //Cursor
        let mut rect = Rect::new(self.text_area.x + line_len_px(self.font_size, self.win().cursor_pos.x as usize) + self.padding.x,
             self.text_area.y + self.font_size * self.win().cursor_pos.y + self.padding.y, 2.0, self.font_size);
        rect.x -= self.window_offset.x;
        rect.y -= self.window_offset.y;

        let rect_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(),
                                          rect, Color::from_rgb(190, 190, 230))?;

        //Draw off limits bar (Where the line number reside)
        //Has to be before num lines
        let off_limits_rect = Rect::new(self.text_area.x, self.text_area.y, 50.0, self.text_area.h);
        let off_limits_rect_mash = Mesh::new_rectangle(ctx, DrawMode::fill(),
                                                            off_limits_rect, Color::from_rgb(25, 25, 40))?;
        canvas.draw(&off_limits_rect_mash, DrawParam::default());
        //Draw line numbers
        for (i, _) in self.win().lines.iter().enumerate() {

            let col = Color::from(match i == self.win().cursor_pos.y as usize{
                false => (85, 85, 100),
                _ => (225, 225, 225), 
            });

            let line_num = Text::new(ggez::graphics::TextFragment {
                text: (i + 1).to_string(),
                color: Some(col),
                font: Some("LiberationMono-Regular".into()),
                scale: Some(ggez::graphics::PxScale::from(self.font_size)),
            });

            canvas.draw(&line_num, Vec2::new(self.text_area.x + self.padding.x,
                                        self.text_area.y + self.font_size * i as f32 - self.window_offset.y + self.padding.y));
        }

        //Draw page slider - Not in use
        //let page_slider_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(),
        //                                            self.slider_rect, Color::from_rgb(120, 120, 120))?;
                                                                        
        //canvas.draw(&page_slider_mesh, DrawParam::default());
        canvas.draw(&rect_mesh, DrawParam::default());
        
        //Explorer backdrop
        let explorer_rect = Rect::new(0.0, self.text_area.y, self.text_area.x, self.text_area.h);
        let explorer_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(),
                                                        explorer_rect, Color::from_rgb(15, 15, 30))?;
        
        canvas.draw(&explorer_mesh, DrawParam::default());
        
        //Explorer
        //Cwd text
        let mut cwd = match env::current_dir()?.to_str() {
            Some(wd) => {
                let spl = wd.split(['/', '\'']);
                spl.last().unwrap().to_owned()
            }
            None => "WD404".to_owned(),
        };

        cwd.insert_str(0, "cwd: ./"); //O(1)
        cwd = cwd.to_ascii_uppercase();

        let cwd_text = Text::new(ggez::graphics::TextFragment {
                text: cwd,
                color: Some(Color::from_rgb(190, 190, 230)),
                font: Some("LiberationMono-Regular".into()),
                scale: Some(ggez::graphics::PxScale::from(self.font_size - 6.0)),
        });

        
        //Draw text for the directory explorer

        let mut i_pair: (u32, u32) = (0, 0);
        for direc in self.top_level_directory.iter_mut() {
            direc.draw_canvas(&mut canvas, self.font_size - 6.0, &mut i_pair, Vec2::new(15.0, 20.0))?;
        }
        
        canvas.draw(&cwd_text, Vec2::new(5.0, self.text_area.y));

        //Draw top toolbar
        let toolbar_rect = Rect::new(0.0, 0.0, ctx.gfx.window().inner_size().width as f32, self.text_area.y);
        let toolbar_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(),
                                     toolbar_rect, Color::from_rgb(15, 15, 30))?;

        canvas.draw(&toolbar_mesh, DrawParam::default());



        canvas.finish(ctx)?;


        //Does not work fully, only caps to ~97fps, why???
        //println!("{}", ctx.time.fps());
        ggez::timer::sleep(Duration::from_micros(((1.0 / 60.0 - 1.0 / ctx.time.fps()) * 1000000.0) as u64)); // Cap to 60fps

        Ok(())
    }
}

pub fn main() -> GameResult {
    let mut cb = ggez::ContextBuilder::new("Vio", "Robert West");

    cb = cb.window_setup(WindowSetup {
        title: "New file - Vio".to_string(),
        samples: ggez::conf::NumSamples::One,
        vsync: false,
        icon: "".to_string(),
        srgb: true
    });
    cb = cb.window_mode(WindowMode {
        width: 1920.0,
        height: 1080.0,
        maximized: true,
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
    let state = MainState::new(&ctx)?;
    event::run(ctx, event_loop, state)
}
