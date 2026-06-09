use std::collections::HashMap;

use raylib:: prelude::*;

const GRID_SIZE_PX: i32          = 700;
const GRID_SIZE: usize           = 15;
const SIDEBAR_SIZE_PX: i32       = 200;
const WINDOW_WIDTH: i32          = GRID_SIZE_PX + SIDEBAR_SIZE_PX;
const WINDOW_HEIGHT: i32         = GRID_SIZE_PX;
const CIRCLE_COUNT: i32          = 15;
const CIRCLE_RADIUS: f32         = 10.0;
const WIDGETS_PAD: i32           = 20;
const INLINE_PAD: i32            = 5;
const PLAYER_COLOR_REC_SIZE: i32 = 20;
const FONT_SIZE: i32             = 20;

#[derive(Debug, Clone)]
struct Game {
    state: GameState,

    cs: Vec<Vec<Circle>>,

    players: Vec<Player>,
    player_turn_idx: Option<usize>,
    player_colors: [(Color, Color); 8],

    drag_l: Option<DraggingLine>,
    conns: Vec<Line>,
    squares: Vec<Square>,

    dt: f32,
    timer: f32,

    alerts: AlertsMap,
}

impl Game {
    fn new() -> Self{
        Game{
            state: GameState::StartScreen,

            cs: {
                let mut cs: Vec<Vec<Circle>> = Vec::new();
                let grid_size = CIRCLE_COUNT as usize;
                let spacing = (WINDOW_WIDTH.min(WINDOW_HEIGHT) as f32)
                    / (CIRCLE_COUNT as f32);
                // let spacing = 500.0/CIRCLE_COUNT as f32;

                for i in 0..grid_size {
                    let mut row = Vec::new();

                    for j in 0..grid_size {
                        row.push(Circle {
                            pos: Vector2 {
                                x: i as f32 * spacing + CIRCLE_RADIUS * 2.0,
                                y: j as f32 * spacing + CIRCLE_RADIUS * 2.0,
                            },
                            grid_x: i,
                            grid_y: j,
                            r: CIRCLE_RADIUS,
                        });
                    }

                    cs.push(row);
                }

                cs
            },

            players: vec![],
            player_turn_idx: None,
            player_colors: [
                // 1. Vibrant Coral / Soft Coral
                (Color::from_hex("D35400").unwrap(), Color::from_hex("E67E22").unwrap()),
                // 2. Amber Orange / Pastel Yellow-Orange
                (Color::from_hex("D4AC0D").unwrap(), Color::from_hex("F4D03F").unwrap()),
                // 3. Lime Green / Soft Green
                (Color::from_hex("27AE60").unwrap(), Color::from_hex("58D68D").unwrap()),
                // 4. Deep Teal / Fresh Mint
                (Color::from_hex("16A085").unwrap(), Color::from_hex("48C9B0").unwrap()),
                // 5. Electric Blue / Sky Blue
                (Color::from_hex("2980B9").unwrap(), Color::from_hex("5DADE2").unwrap()),
                // 6. Royal Violet / Lavender Blue
                (Color::from_hex("76448A").unwrap(), Color::from_hex("AF7AC5").unwrap()),
                // 7. Vivid Magenta / Orchid Pink
                (Color::from_hex("9B59B6").unwrap(), Color::from_hex("C39BD3").unwrap()),
                // 8. Dark Rose / Pastel Pink
                (Color::from_hex("C0392B").unwrap(), Color::from_hex("EC7063").unwrap()),
            ],

            drag_l: None,
            conns: vec![],
            squares: vec![],
            dt: 0.0,
            timer: 0.0,
            alerts: HashMap::new()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    StartScreen,
    Running,
    EndScreen,
}

#[derive(Debug, Clone, PartialEq)]
struct Player {
    name: String,
    color: (Color, Color),
    square_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
struct Circle {
    pos: Vector2,
    grid_x: usize,
    grid_y: usize,
    r: f32,
}

#[derive(Debug, Clone)]
struct DraggingLine {
    start: Circle,
    end: Option<Circle>,
    pointer: Vector2,
}

#[derive(Debug, Clone, PartialEq)]
struct Line {
    start: Circle,
    end: Circle,
    player: Player,
}

#[derive(Debug, Clone)]
struct Square {
    x: usize,
    y: usize,
    player: Player,
}
fn same_pos(a: &Circle, x: usize, y: usize) -> bool {
    a.grid_x == x && a.grid_y == y
}

fn line_exists(conns: &[Line], ax: usize, ay: usize, bx: usize, by: usize) -> bool {
    conns.iter().any(|line| {
        (same_pos(&line.start, ax, ay) && same_pos(&line.end, bx, by))
            || (same_pos(&line.start, bx, by) && same_pos(&line.end, ax, ay))
    })
}

fn square_complete(conns: &[Line], x: usize, y: usize) -> bool {
    line_exists(conns, x, y, x + 1, y)
        && line_exists(conns, x, y, x, y + 1)
        && line_exists(conns, x + 1, y, x + 1, y + 1)
        && line_exists(conns, x, y + 1, x + 1, y + 1)
}

fn add_square_if_complete(
    conns: &[Line],
    squares: &mut Vec<Square>,
    x: usize,
    y: usize,
    grid_size: usize,
    player: &mut Player,
) {
    if x >= grid_size - 1 || y >= grid_size - 1 {
        return;
    }

    let already_exists = squares.iter().any(|s| s.x == x && s.y == y);

    if !already_exists && square_complete(conns, x, y) {
        squares.push(Square { x, y, player: player.clone()});
        player.square_count += 1;
    }
}

fn are_adjacent(c1: Circle, c2: Circle) -> bool {
    let dx = c1.grid_x.abs_diff(c2.grid_x);
    let dy = c1.grid_y.abs_diff(c2.grid_y);

    dx + dy == 1
}

type AlertsMap = HashMap<Alert, bool>;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Alert {
    NoPlayer,
    WrongMove,
    PlayerLimitExceeded,
}

fn update_alerts(timer: &mut f32, dt: f32, map: &mut AlertsMap){
    for (_name, status) in map.iter_mut() {
        if *status {
            *timer += dt;
        }
        if *timer > 3.0 && *status {
            *timer = 0.0;
            *status = false;
        }
    }
}

fn show_alerts(d: &mut RaylibDrawHandle, map: &AlertsMap){
    map
        .iter()
        .for_each(|(name, status)| {
            if *status{
                let r = Rectangle{
                    x: WINDOW_WIDTH as f32/4.0,
                    y: 0.0,
                    width: WINDOW_WIDTH as f32/2.0,
                    height: WINDOW_HEIGHT as f32/16.0,
                };
                d.draw_rectangle_rec(r, Color::WHITE);
                d.draw_rectangle_lines_ex(r, 3.0, Color::RED);
                match name{
                    Alert::WrongMove => d.draw_text("Alert: Wrong move!", (r.x as i32) + WINDOW_WIDTH/32, WINDOW_HEIGHT/32, 20, Color::RED),
                    Alert::NoPlayer => d.draw_text("Alert: No Player selected!", (r.x as i32) + WINDOW_WIDTH/32, WINDOW_HEIGHT/32, 20, Color::RED),
                    Alert::PlayerLimitExceeded => d.draw_text("Alert: Player limit exceeded!", (r.x as i32) + WINDOW_WIDTH/32, WINDOW_HEIGHT/32, 20, Color::RED),
                }
            }
        });
}

fn check_collision_mouse_grid(rl: &RaylibHandle) -> bool {
    check_collision_point_poly(rl.get_mouse_position(), &[
        Vector2{x: 0.0, y: 0.0},
        Vector2{x: GRID_SIZE_PX as f32, y: 0.0},
        Vector2{x: GRID_SIZE_PX as f32, y: GRID_SIZE_PX as f32},
        Vector2{x: 0.0, y: GRID_SIZE_PX as f32},
    ])
}

fn is_line_taken(lines: &[Line], start: &Circle, end: &Circle) -> bool{
    lines.iter().any(|l| {
        (l.start == *start && l.end == *end) || (l.start == *end && l.end == *start)
    })
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("square game")
        .build();

    rl.set_target_fps(60);

    let mut game = Game::new();
    while !rl.window_should_close(){
        render_running_state(&mut game, &mut rl, &thread);
    }
}

fn render_running_state(g:&mut Game, rl: &mut RaylibHandle, thread: &RaylibThread){
        g.dt = rl.get_frame_time();
        update_alerts(&mut g.timer, g.dt, &mut g.alerts);

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if g.player_turn_idx.is_none() && check_collision_mouse_grid(rl){
                g.alerts.insert(Alert::NoPlayer, true);
            } else {
                let mouse = rl.get_mouse_position();

                'outer: for row in &g.cs {
                    for c in row {
                        if check_collision_point_circle(mouse, c.pos, c.r * 1.7) {
                            g.drag_l = Some(DraggingLine {
                                start: c.clone(),
                                pointer: mouse,
                                end: None,
                            });

                            break 'outer;
                        }
                    }
                }
            }
        }

        if rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse = rl.get_mouse_position();

            if let Some(l) = &mut g.drag_l {
                l.pointer = mouse;

                for row in &g.cs {
                    for c in row {
                        if check_collision_point_circle(mouse, c.pos, c.r * 1.7) {
                            l.end = Some(c.clone());
                        }
                    }
                }
            }
        }

        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) && let Some(p_idx) = g.player_turn_idx{
                let players_len = g.players.clone().len();
                let active_player = &mut g.players[p_idx];
                if let Some(DraggingLine { start, end: Some(end), .. }) = g.drag_l.take()
                    && are_adjacent(start.clone(), end.clone())
                    && !is_line_taken(&g.conns, &start, &end)
                {
                    let new_line = Line {
                        start: start.clone(),
                        end: end.clone(),
                        player: active_player.clone()
                    };

                    g.conns.push(new_line);

                    g.player_turn_idx = match g.player_turn_idx{
                        Some(i) => {
                            if i == players_len - 1{
                                Some(0)
                            } else {
                                Some(i + 1)
                            }
                        }
                        None => Some(0)
                    };

                    let sx = start.grid_x;
                    let sy = start.grid_y;
                    let ex = end.grid_x;
                    let ey = end.grid_y;

                    if sy == ey {
                        let x = sx.min(ex);
                        let y = sy;

                        add_square_if_complete(&g.conns, &mut g.squares, x, y, GRID_SIZE, active_player);

                        if y > 0 {
                            add_square_if_complete(&g.conns, &mut g.squares, x, y - 1, GRID_SIZE, active_player);
                        }
                    }

                    if sx == ex {
                        let x = sx;
                        let y = sy.min(ey);

                        add_square_if_complete(&g.conns, &mut g.squares, x, y, GRID_SIZE, active_player);

                        if x > 0 {
                            add_square_if_complete(&g.conns, &mut g.squares, x - 1, y, GRID_SIZE, active_player);
                        }
                    }
                } else if !check_collision_mouse_grid(rl){
                    g.drag_l = None;
                } else {
                    g.alerts.insert(Alert::WrongMove, true);
                    g.drag_l = None;
                }
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(Color::WHITE);

        //draw squares
        for s in &g.squares {
            let top_left = g.cs[s.x][s.y].pos;
            let bottom_right = g.cs[s.x + 1][s.y + 1].pos;

            let rect = Rectangle {
                x: top_left.x,
                y: top_left.y,
                width: bottom_right.x - top_left.x,
                height: bottom_right.y - top_left.y,
            };

            d.draw_rectangle_rec(rect, s.player.color.1);
        }
        //draw possible circles
        for row in &g.cs {
            for c in row {
                let is_adjacent_to_start = g.drag_l
                    .as_ref()
                    .is_some_and(|l| are_adjacent(l.start.clone(), c.clone()));

                let color = if is_adjacent_to_start {
                    Color::new(0x33, 0xFF, 0x33, 0xFF)
                } else {
                    Color::new(0x77, 0x77, 0x77, 0xFF)
                };

                d.draw_circle_v(c.pos, c.r, color);
            }
        }


        //draw circle grid
        for c in &g.conns{
            d.draw_line_ex(c.start.pos, c.end.pos, 3.0, c.player.color.0);
            d.draw_circle_v(c.start.pos, c.start.r, Color::GRAY);
            d.draw_circle_v(c.end.pos, c.end.r, Color::GRAY);
        }

        //draw dragging line
        if let Some(l) = g.drag_l.clone(){
            d.draw_line_ex(l.start.pos, l.pointer, 5.0, Color::RED);
            d.draw_circle_v(l.start.pos, l.start.r, Color::RED);
            if let Some(end_l) = l.end {
                d.draw_circle_v(end_l.pos, end_l.r, Color::RED);
            }
        }

        //draw end game button
        let button_rect = Rectangle{
            x: (WINDOW_WIDTH - 70) as f32,
            y: (WINDOW_HEIGHT - 40) as f32,
            width: 60.0,
            height: 30.0,
        };
        if d.gui_button(button_rect, "End Game") {
            g.state = GameState::EndScreen;
        }

        //draw add player button
        let button_rect = Rectangle{
            x: (GRID_SIZE_PX + WIDGETS_PAD) as f32,
            y: 10.0,
            width: (SIDEBAR_SIZE_PX - WIDGETS_PAD * 2) as f32,
            height: 40.0,
        };
        if d.gui_button(button_rect, "Add player") {
            if g.players.len() > 7 {
                g.alerts.insert(Alert::PlayerLimitExceeded, true);
            } else {
                g.player_turn_idx = match g.player_turn_idx{
                    Some(i) => { Some(i) }
                    None => Some(0)
                };
                g.players.push(Player{
                    name: format!("player {}", g.players.len() + 1),
                    color: g.player_colors[g.players.len()],
                    square_count: 0_usize,
                });
            }
        }

        //draw players colors
        g.players
            .iter()
            .enumerate()
            .for_each(|(i, p)| {
                let y_pos = (60 + i * 30) as i32;

                let counter_text = &p.square_count.to_string();
                let counter_size = d.measure_text(counter_text, FONT_SIZE);

                d.draw_text     (counter_text, 720, y_pos, FONT_SIZE, Color::BLACK);
                d.draw_rectangle(counter_size + INLINE_PAD + 720, y_pos, PLAYER_COLOR_REC_SIZE, PLAYER_COLOR_REC_SIZE, p.color.0);
                d.draw_text     (&p.name, counter_size + PLAYER_COLOR_REC_SIZE + INLINE_PAD * 2 + 720, y_pos, FONT_SIZE, Color::BLACK);
            });

        //draw current turn player indicator
        if let Some(p_idx)= g.player_turn_idx{
            // d.draw_circle(750, (70 + p_idx * 30) as i32, 5.0, Color::BLACK);
            let rec = Rectangle{
                x: 715.0,
                y: (55 + p_idx * 30) as f32,
                width: (65 + d.measure_text(&g.players[p_idx].name, 20)) as f32,
                height: 30.0,
            };
            d.draw_rectangle_lines_ex(rec, 2.0, Color::BLACK);
        }

        //blank board when no players
        if g.player_turn_idx.is_none(){
            d.draw_rectangle(0, 0, GRID_SIZE_PX, GRID_SIZE_PX, Color::new(0xFF, 0xFF, 0xFF, 0xAA));
        }

        show_alerts(&mut d, &g.alerts);
}
