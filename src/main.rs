use raylib:: prelude::*;

const GRID_SIZE_PX: i32 = 700;
const SIDEBAR_SIZE_PX: i32 = 200;
const WINDOW_WIDTH: i32 = GRID_SIZE_PX + SIDEBAR_SIZE_PX;
const WINDOW_HEIGHT: i32 = GRID_SIZE_PX;
const CIRCLE_COUNT: i32 = 15;
const CIRCLE_RADIUS: f32 = 10.0;
const WIDGETS_PAD: i32 = 20;

#[derive(Debug, Clone, PartialEq)]
struct Player {
    color: (Color, Color)
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
    player: Player,
) {
    if x >= grid_size - 1 || y >= grid_size - 1 {
        return;
    }

    let already_exists = squares.iter().any(|s| s.x == x && s.y == y);

    if !already_exists && square_complete(conns, x, y) {
        squares.push(Square { x, y, player});
    }
}

fn are_adjacent(c1: Circle, c2: Circle) -> bool {
    let dx = c1.grid_x.abs_diff(c2.grid_x);
    let dy = c1.grid_y.abs_diff(c2.grid_y);

    dx + dy == 1
}

#[derive(Clone, Debug)]
struct Alert {
    active: bool,
    description: String,
}

fn update_alert(timer: &mut f32, dt: f32, alert: &mut Alert){
    if alert.active{
        *timer += dt;
    }
    if *timer > 3.0 && alert.active {
        *timer = 0.0;
        alert.active = false;
    }
}

fn show_alert(d: &mut RaylibDrawHandle, alert: Alert){
    if alert.active {
        let r = Rectangle{
            x: WINDOW_WIDTH as f32/4.0,
            y: 0.0,
            width: WINDOW_WIDTH as f32/2.0,
            height: WINDOW_HEIGHT as f32/16.0,
        };
        d.draw_rectangle_rec(r, Color::WHITE);
        d.draw_rectangle_lines_ex(r, 3.0, Color::RED);
        d.draw_text(&format!("Alert: {}!", alert.description), (r.x as i32) + WINDOW_WIDTH/32, WINDOW_HEIGHT/32, 20, Color::RED);
    }
}

fn check_collision_mouse_grid(rl: &RaylibHandle) -> bool {
    check_collision_point_poly(rl.get_mouse_position(), &[
        Vector2{x: 0.0, y: 0.0},
        Vector2{x: GRID_SIZE_PX as f32, y: 0.0},
        Vector2{x: GRID_SIZE_PX as f32, y: GRID_SIZE_PX as f32},
        Vector2{x: 0.0, y: GRID_SIZE_PX as f32},
    ])
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .title("square game")
        .build();

    rl.set_target_fps(60);

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

    let mut players: Vec<Player> = vec![];
    let mut player_turn_idx: Option<usize> = None;
    let mut player_colors = [
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
    ].into_iter().cycle();

    let mut drag_l: Option<DraggingLine> = None;
    let mut conns: Vec<Line> = vec![];
    let mut squares: Vec<Square> = vec![];

    let mut dt: f32;
    let mut timer: f32 = 0.0;
    let mut alert_wrong_move = Alert{
        active: false,
        description: "Wrong move".to_string(),
    };
    let mut alert_no_player = Alert{
        active: false,
        description: "No Player added".to_string(),
    };

    while !rl.window_should_close() {
        dt = rl.get_frame_time();
        update_alert(&mut timer, dt, &mut alert_wrong_move);
        update_alert(&mut timer, dt, &mut alert_no_player);


        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            if player_turn_idx.is_none() && check_collision_mouse_grid(&rl){
                alert_no_player.active = true;
            } else {
                let mouse = rl.get_mouse_position();

                'outer: for row in &cs {
                    for c in row {
                        if check_collision_point_circle(mouse, c.pos, c.r * 1.7) {
                            drag_l = Some(DraggingLine {
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

            if let Some(l) = &mut drag_l {
                l.pointer = mouse;

                for row in &cs {
                    for c in row {
                        if check_collision_point_circle(mouse, c.pos, c.r * 1.7) {
                            l.end = Some(c.clone());
                        }
                    }
                }
            }
        }

        if rl.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) && let Some(p_idx) = player_turn_idx{
                let active_player = players[p_idx].clone();
                if let Some(DraggingLine { start, end: Some(end), .. }) = drag_l.take()
                    && are_adjacent(start.clone(), end.clone())
                        && ! conns.iter().any(|l| *l == Line{start: start.clone(), end: end.clone(), player: active_player.clone()})
                {
                    let new_line = Line {
                        start: start.clone(),
                        end: end.clone(),
                        player: active_player.clone()
                    };

                    conns.push(new_line);

                    player_turn_idx = match player_turn_idx{
                        Some(i) => {
                            if i == players.len() - 1{
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

                        add_square_if_complete(&conns, &mut squares, x, y, grid_size, active_player.clone());

                        if y > 0 {
                            add_square_if_complete(&conns, &mut squares, x, y - 1, grid_size, active_player.clone());
                        }
                    }

                    if sx == ex {
                        let x = sx;
                        let y = sy.min(ey);

                        add_square_if_complete(&conns, &mut squares, x, y, grid_size, active_player.clone());

                        if x > 0 {
                            add_square_if_complete(&conns, &mut squares, x - 1, y, grid_size, active_player.clone());
                        }
                    }
                } else if player_turn_idx.is_none() && !check_collision_mouse_grid(&rl){
                    drag_l = None;
                } else {
                    alert_wrong_move.active = true;
                    drag_l = None;
                }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        for s in &squares {
            let top_left = cs[s.x][s.y].pos;
            let bottom_right = cs[s.x + 1][s.y + 1].pos;

            let rect = Rectangle {
                x: top_left.x,
                y: top_left.y,
                width: bottom_right.x - top_left.x,
                height: bottom_right.y - top_left.y,
            };

            d.draw_rectangle_rec(rect, s.player.color.1);
        }
        for row in &cs {
            for c in row {
                let is_adjacent_to_start = drag_l
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

        if let Some(l) = drag_l.clone(){
            d.draw_line_ex(l.start.pos, l.pointer, 5.0, Color::RED);
            d.draw_circle_v(l.start.pos, l.start.r, Color::RED);
            if let Some(end_l) = l.end {
                d.draw_circle_v(end_l.pos, end_l.r, Color::RED);
            }
        }

        for c in &conns{
            d.draw_line_ex(c.start.pos, c.end.pos, 3.0, c.player.color.0);
            d.draw_circle_v(c.start.pos, c.start.r, Color::GRAY);
            d.draw_circle_v(c.end.pos, c.end.r, Color::GRAY);
        }

        show_alert(&mut d, alert_wrong_move.clone());
        show_alert(&mut d, alert_no_player.clone());

        let button_rect = Rectangle{
            x: (GRID_SIZE_PX + WIDGETS_PAD) as f32,
            y: 10.0,
            width: (SIDEBAR_SIZE_PX - WIDGETS_PAD * 2) as f32,
            height: 40.0,
        };
        if d.gui_button(button_rect, "Add player") {
            players.push(Player{
                color: player_colors.next().unwrap(),
            });
            player_turn_idx = match player_turn_idx{
                Some(i) => { Some(i) }
                None => Some(0)
            };
        }

        players.iter().enumerate().for_each(|(i, p)| d.draw_rectangle(720, (60 + i * 30) as i32, 20, 20, p.color.0));
        if let Some(p_idx)= player_turn_idx{
            d.draw_circle(750, (70 + p_idx * 30) as i32, 5.0, Color::BLACK);
        }

    }
}
