use std::{
    env,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use image::Rgb;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap, Widget, StatefulWidget},
    Frame, Terminal, style::Style, prelude::Rect,
};
use ratatui_image::{
    picker::Picker,
    protocol::{ImageSource, ResizeProtocol},
    Resize, ResizeImage,
};

struct App {
    pub filename: String,
    pub picker: Picker,
    pub image_source: ImageSource,
    pub image_state: Box<dyn ResizeProtocol>,
    pub mask: Mask,
    pub progress: MaskState,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = env::args()
        .nth(1)
        .unwrap_or("/home/yly/out.png".to_string());

    let image = image::io::Reader::open(&filename)?.decode()?;

    let mut picker = Picker::from_termios(Some(Rgb::<u8>([255, 0, 255])))?;

    let image_source = ImageSource::new(image.clone(), picker.font_size());
    let image_state = picker.new_state(image);

    let mut app = App {
        filename,
        picker,
        image_source,
        image_state,
        mask: Mask {  },
        progress: MaskState { progress: 0.0 },
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(50);
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        let mut prog: f32 = 1.0 / (terminal.size().unwrap().width as f32 * terminal.size().unwrap().height as f32) as f32;
        // prog *= rand::random::<f32>() +1.0;
        app.progress.progress = (app.progress.progress + prog).clamp(0.0, 1.0);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(c) => match c {
                            'q' => break,
                            ' ' => {
                                app.picker.cycle_protocols();
                                app.image_state =
                                    app.picker.new_state(app.image_source.image.clone());
                            }
                            _ => {}
                        },
                        KeyCode::Esc => break,
                        KeyCode::Enter => (),
                        _ => {}
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(25), Constraint::Min(1)].as_ref())
        .split(f.size());

    // let block_top = Block::default()
    //     .borders(Borders::ALL)
    //     .title("ratatui-image");
    let dyn_img = &app.image_source.image;
    let style = ratatui::style::Style::default().fg(ratatui::style::Color::Green);
    let lines = vec![
        Line::styled("丟您圓「停冬像個眼燈馬夕」而燈星；奶內信次許蛋婆法！法冬就爸母品嗎羽", style),
        Line::styled("進吃每種反王後「鼻冬我牙定亮」歌今法早士它千：汗愛笑穴因有躲兔開喝面", style),
        Line::styled("至苗清幾安抄。習正多習東師抱時飯路能麻汗把母，耍字身丟；呀車這火就耳", style),
        Line::styled("這娘掃東向。唱美果巾姊貝河念月色書汗？道告西打雄喝百，里怕綠司更卜丁", style),
        Line::styled("兆旁門活，自加請拍幾停息？瓜封像師斗干反光；結裝尼真只早兆樹花節飛犬", style),
        Line::styled("聽呢點扒像就早造。黑雞跟這主院科工又叫登昌彩喜兌抄急意肖？木四這，蛋", style),
        Line::styled("長老尼常天念珠背穿間動叫掃在友：他四者以快成麻「示活誰才貫真細樹黃個", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("在做尾點相央。屋口拉干旁泉具苦什往長過封條畫向勿民以屋；立貫亮從木山", style),
        Line::styled("媽貝息長好京還三心聽辛。", style)
    ];
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        chunks[0],
    );

    let block_bottom = Block::default().borders(Borders::ALL).title(format!("{}",app.progress.progress));
    let image = ResizeImage::new(None).resize(Resize::Fit);
    let mask = Mask{};
    f.render_stateful_widget(image, block_bottom.inner(chunks[1]), &mut app.image_state);
    f.render_widget(block_bottom, chunks[1]);
    let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints(vec![Constraint::Length(4),Constraint::Min(2)])
    .split(f.size());
    f.render_stateful_widget(mask,layout[1] , &mut app.progress);
}
struct Mask {}
struct MaskState {
    pub progress:f32,
}
impl StatefulWidget for Mask {
    type State = MaskState;

    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer, state: &mut Self::State) {
        // area should always be full frame
        // let start = f32::floor(state.progress * area.width as f32 * area.height as f32) as usize;
        let mut start_x = 0;
        let mut start_y = 0;
        let width = area.width;
        let height = area.height;
        while ((start_y*width + start_x) as f32/ (area.area()) as f32) < state.progress {
            if start_x + 1 >= width {
                start_x = 0;
                start_y += 1;
            } else {
                start_x += 1;
            }
            if start_y +1 == height && start_x + 1 >= width {
                break;
            }
        }
        // try_skip, must do before resetting cells
        for i in start_x..width {
            if buf.get(i+area.x, start_y+area.y).symbol.eq(" "){
                let mut diff = 1.0 / (area.width as f32 * area.height as f32) as f32;
                diff *= 0.5;
                state.progress += diff;
            }
        }
        let len = buf.content.len();
        for index in 0..len {
            let should_clear:bool;
            let (x,y) = buf.pos_of(index);
            should_clear = (!is_in(x,y,start_x,start_y)) && area.intersects(Rect::new(x+area.x,y+area.y,1,1));
            if should_clear {
                if buf.get(x+area.x, y+area.y).symbol.eq(" "){
                    continue;
                } else {
                    buf.get_mut(x+area.x, y+area.y).reset();
                    buf.get_mut(x+area.x, y+area.y).set_skip(false);
                }  
            }
        }
        fn is_in(x:u16,y:u16,x_:u16,y_:u16)-> bool {
            // if this is true, should NOT clear;
            if y < y_ {
                return true;
            } else if y==y_{
                return x<=x_;
            } else {
                return false;
            }
        }
    }
}