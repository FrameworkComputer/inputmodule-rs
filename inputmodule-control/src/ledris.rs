//! Ledris, a falling block puzzle: blocks of four fall into the well, move and turn
//! them so they fill whole rows, which then disappear. It gets faster every ten rows,
//! and is over when the blocks pile up to the top.
//!
//! The next block is shown above the well. The falling block glides down smoothly
//! between rows, and a dim copy shows where it will land.

use crossterm::event::KeyCode;
use rand::prelude::*;

use crate::hostgame::{add_rect, dark, play, to_grid, Burst, HostGame, Input, Light};
use crate::inputmodule::{GreyGrid, HEIGHT};

/// The next block is shown in the rows above the well, separated by a dashed line
const SEPARATOR_ROW: usize = 2;
const WELL_TOP: usize = SEPARATOR_ROW + 1;
const WELL_H: usize = HEIGHT - WELL_TOP;

const PIECE_BRIGHTNESS: f32 = 1.0;
const BLOCK_BRIGHTNESS: f32 = 0.45;
const GHOST_BRIGHTNESS: f32 = 0.12;
const NEXT_BRIGHTNESS: f32 = 0.35;
const SEPARATOR_BRIGHTNESS: f32 = 0.15;

/// How fast blocks fall, in rows per second. Faster with every level.
const START_GRAVITY: f32 = 2.5;
const GRAVITY_PER_LEVEL: f32 = 1.25;
const MAX_GRAVITY: f32 = 40.0;
const LINES_PER_LEVEL: u32 = 10;
/// How long a block can still be moved after it lands, and how often moving
/// it starts that time over, so it can't be moved around forever
const LOCK_DELAY: f32 = 0.5;
const MAX_LOCK_RESETS: u32 = 15;

/// How long full rows flash before they disappear
const CLEAR_SECS: f32 = 0.3;
const OVER_SECS: f32 = 1.0;

/// Points for clearing 1 to 4 rows at once, times the level plus one
const LINE_POINTS: [u32; 4] = [100, 300, 500, 800];

/// The seven blocks: the size of the square they turn in, and their cells in it
const SHAPES: [(i32, [(i32, i32); 4]); 7] = [
    (4, [(0, 1), (1, 1), (2, 1), (3, 1)]), // I
    (2, [(0, 0), (1, 0), (0, 1), (1, 1)]), // O
    (3, [(1, 0), (0, 1), (1, 1), (2, 1)]), // T
    (3, [(1, 0), (2, 0), (0, 1), (1, 1)]), // S
    (3, [(0, 0), (1, 0), (1, 1), (2, 1)]), // Z
    (3, [(0, 0), (0, 1), (1, 1), (2, 1)]), // J
    (3, [(2, 0), (0, 1), (1, 1), (2, 1)]), // L
];

/// Where to try to put a turned block if it doesn't fit where it is,
/// like when it's against a wall. Negative y is up.
const KICKS: [(i32, i32); 6] = [(0, 0), (-1, 0), (1, 0), (0, -1), (-2, 0), (2, 0)];

#[derive(Clone, Copy, Debug, PartialEq)]
struct Piece {
    kind: usize,
    /// Quarter turns clockwise
    rot: usize,
    /// Top left of the square it turns in, in cells of the well
    x: i32,
    y: i32,
}

impl Piece {
    /// A new block at the top in the middle
    fn spawn(kind: usize, width: usize) -> Self {
        let (size, cells) = SHAPES[kind];
        let top = cells.iter().map(|&(_, y)| y).min().unwrap();
        Self {
            kind,
            rot: 0,
            x: (width as i32 - size) / 2,
            y: -top,
        }
    }

    /// Column and row of each cell in the well
    fn cells(self) -> [(i32, i32); 4] {
        let (size, cells) = SHAPES[self.kind];
        cells.map(|(mut x, mut y)| {
            for _ in 0..self.rot % 4 {
                (x, y) = (size - 1 - y, x);
            }
            (self.x + x, self.y + y)
        })
    }

    fn moved(self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..self
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum State {
    /// Waiting for the first key press
    Ready,
    Playing,
    /// Full rows flash before they disappear
    Clearing {
        since: f32,
    },
    Over {
        since: f32,
    },
}

#[derive(Clone)]
struct Game {
    rng: ThreadRng,
    /// Screen width in LEDs, one or more LED matrices side by side
    width: usize,
    state: State,
    /// Cells of blocks that landed, indexed [row][column], row 0 at the top of the well
    well: Vec<Vec<bool>>,
    piece: Piece,
    next: usize,
    /// The next blocks, each of the seven once in random order, so none is missing for long
    bag: Vec<usize>,
    /// How far the block has fallen towards the next row, to draw it in between
    fall: f32,
    /// How long the block has been lying on something
    lock_time: f32,
    lock_resets: u32,
    /// Lowest row the block reached, getting lower allows moving it longer again
    lowest: i32,
    /// Full rows that are flashing
    clearing: Vec<usize>,
    score: u32,
    lines: u32,
    best: u32,
    burst: Option<Burst>,
}

impl Game {
    fn new(width: usize) -> Self {
        let mut game = Self {
            rng: thread_rng(),
            width,
            state: State::Ready,
            well: Vec::new(),
            piece: Piece::spawn(0, width),
            next: 0,
            bag: Vec::new(),
            fall: 0.0,
            lock_time: 0.0,
            lock_resets: 0,
            lowest: 0,
            clearing: Vec::new(),
            score: 0,
            lines: 0,
            best: 0,
            burst: None,
        };
        game.restart();
        game
    }

    fn restart(&mut self) {
        self.state = State::Ready;
        self.well = vec![vec![false; self.width]; WELL_H];
        self.bag.clear();
        self.clearing.clear();
        self.score = 0;
        self.lines = 0;
        self.next = self.take_from_bag();
        self.spawn(0.0);
        self.state = State::Ready;
    }

    fn take_from_bag(&mut self) -> usize {
        if self.bag.is_empty() {
            self.bag = (0..SHAPES.len()).collect();
            self.bag.shuffle(&mut self.rng);
        }
        self.bag.pop().unwrap()
    }

    fn level(&self) -> u32 {
        self.lines / LINES_PER_LEVEL
    }

    /// Rows per second
    fn gravity(&self) -> f32 {
        (START_GRAVITY * GRAVITY_PER_LEVEL.powi(self.level() as i32)).min(MAX_GRAVITY)
    }

    /// Inside the well and not on any block. Above the top is free.
    fn fits(&self, piece: Piece) -> bool {
        piece.cells().iter().all(|&(x, y)| {
            x >= 0
                && (x as usize) < self.width
                && y < WELL_H as i32
                && (y < 0 || !self.well[y as usize][x as usize])
        })
    }

    fn landing(&self) -> Piece {
        let mut piece = self.piece;
        while self.fits(piece.moved(0, 1)) {
            piece = piece.moved(0, 1);
        }
        piece
    }

    fn spawn(&mut self, now: f32) {
        self.piece = Piece::spawn(self.next, self.width);
        self.next = self.take_from_bag();
        self.fall = 0.0;
        self.lock_time = 0.0;
        self.lock_resets = 0;
        self.lowest = self.piece.y;
        self.state = State::Playing;
        if !self.fits(self.piece) {
            self.game_over(now);
        }
    }

    fn game_over(&mut self, now: f32) {
        self.state = State::Over { since: now };
        self.best = self.best.max(self.score);
        let cells = self.piece.cells();
        let x = cells.iter().map(|&(x, _)| x as f32 + 0.5).sum::<f32>() / 4.0;
        let y = cells.iter().map(|&(_, y)| y as f32 + 0.5).sum::<f32>() / 4.0;
        self.burst = Some(Burst::new(&mut self.rng, x, WELL_TOP as f32 + y.max(0.5)));
    }

    /// The player moved or turned the block. If it's lying on something,
    /// that gives a bit more time before it locks.
    fn moved(&mut self) {
        if !self.fits(self.piece.moved(0, 1)) && self.lock_resets < MAX_LOCK_RESETS {
            self.lock_time = 0.0;
            self.lock_resets += 1;
        }
    }

    fn shift(&mut self, dx: i32) {
        let moved = self.piece.moved(dx, 0);
        if self.fits(moved) {
            self.piece = moved;
            self.moved();
        }
    }

    fn rotate(&mut self, quarter_turns: usize) {
        let turned = Piece {
            rot: (self.piece.rot + quarter_turns) % 4,
            ..self.piece
        };
        let kicked = KICKS
            .iter()
            .map(|&(dx, dy)| turned.moved(dx, dy))
            .find(|&p| self.fits(p));
        if let Some(piece) = kicked {
            self.piece = piece;
            self.moved();
        }
    }

    /// Down one row, if there's room
    fn step_down(&mut self) -> bool {
        let down = self.piece.moved(0, 1);
        if !self.fits(down) {
            return false;
        }
        self.piece = down;
        if down.y > self.lowest {
            self.lowest = down.y;
            self.lock_time = 0.0;
            self.lock_resets = 0;
        }
        true
    }

    fn soft_drop(&mut self) {
        if self.step_down() {
            self.fall = 0.0;
            self.score += 1;
        }
    }

    fn hard_drop(&mut self, now: f32) {
        while self.step_down() {
            self.score += 2;
        }
        self.lock(now);
    }

    /// The block becomes part of the well
    fn lock(&mut self, now: f32) {
        let mut above_top = false;
        for (x, y) in self.piece.cells() {
            if y < 0 {
                above_top = true;
            } else {
                self.well[y as usize][x as usize] = true;
            }
        }
        if above_top {
            self.game_over(now);
            return;
        }

        self.clearing = (0..WELL_H)
            .filter(|&y| self.well[y].iter().all(|&c| c))
            .collect();
        if self.clearing.is_empty() {
            self.spawn(now);
        } else {
            self.score += LINE_POINTS[self.clearing.len() - 1] * (self.level() + 1);
            self.lines += self.clearing.len() as u32;
            self.state = State::Clearing { since: now };
        }
    }

    /// Take out the full rows, the ones above drop down
    fn remove_full_rows(&mut self) {
        for &y in &self.clearing {
            self.well.remove(y);
            self.well.insert(0, vec![false; self.width]);
        }
        self.clearing.clear();
    }

    fn fall(&mut self, now: f32, dt: f32) {
        if self.fits(self.piece.moved(0, 1)) {
            self.fall += self.gravity() * dt;
            while self.fall >= 1.0 && self.step_down() {
                self.fall -= 1.0;
            }
            if !self.fits(self.piece.moved(0, 1)) {
                self.fall = 0.0;
            }
        } else {
            self.fall = 0.0;
            self.lock_time += dt;
            if self.lock_time >= LOCK_DELAY {
                self.lock(now);
            }
        }
    }
}

/// Light up a cell of the well, at a row that can be in between rows.
/// Nothing above the well, that's where the next block is shown.
fn add_cell(light: &mut Light, x: i32, y: f32, brightness: f32) {
    let top = WELL_TOP as f32 + y;
    let clipped = top.max(WELL_TOP as f32);
    if top + 1.0 > clipped {
        add_rect(
            light,
            (x as f32, clipped),
            (x as f32 + 1.0, top + 1.0),
            brightness,
        );
    }
}

impl HostGame for Game {
    fn player_x(&self) -> f32 {
        self.piece.x as f32
    }

    fn update(&mut self, input: &mut Input, now: f32, dt: f32) {
        match self.state {
            State::Ready => {
                if !input.presses.is_empty() {
                    self.state = State::Playing;
                }
                return;
            }
            State::Over { since } => {
                let again = input
                    .presses
                    .iter()
                    .any(|k| matches!(k, KeyCode::Char(' ') | KeyCode::Enter));
                if again && now - since >= OVER_SECS {
                    self.restart();
                    self.state = State::Playing;
                }
                return;
            }
            State::Clearing { since } => {
                if now - since >= CLEAR_SECS {
                    self.remove_full_rows();
                    self.spawn(now);
                }
                return;
            }
            State::Playing => {}
        }

        for &key in &input.presses {
            match key {
                KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => self.shift(-1),
                KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => self.shift(1),
                KeyCode::Up | KeyCode::Char('x') | KeyCode::Char('w') | KeyCode::Char('k') => {
                    self.rotate(1)
                }
                KeyCode::Char('z') => self.rotate(3),
                KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => self.soft_drop(),
                KeyCode::Char(' ') | KeyCode::Enter => self.hard_drop(now),
                _ => {}
            }
            if self.state != State::Playing {
                return;
            }
        }
        self.fall(now, dt);
    }

    fn render(&self, now: f32) -> GreyGrid {
        let mut light: Light = dark(self.width);

        for x in (0..self.width).step_by(2) {
            add_rect(
                &mut light,
                (x as f32, SEPARATOR_ROW as f32),
                (x as f32 + 1.0, SEPARATOR_ROW as f32 + 1.0),
                SEPARATOR_BRIGHTNESS,
            );
        }

        // The next block, in the middle above the well
        let next = Piece {
            kind: self.next,
            rot: 0,
            x: 0,
            y: 0,
        }
        .cells();
        let min_x = next.iter().map(|&(x, _)| x).min().unwrap();
        let max_x = next.iter().map(|&(x, _)| x).max().unwrap();
        let min_y = next.iter().map(|&(_, y)| y).min().unwrap();
        let offset_x = (self.width as i32 - (max_x - min_x + 1)) / 2 - min_x;
        for (x, y) in next {
            let (x, y) = ((x + offset_x) as f32, (y - min_y) as f32);
            add_rect(&mut light, (x, y), (x + 1.0, y + 1.0), NEXT_BRIGHTNESS);
        }

        // Full rows light up and fade out before they disappear
        let flash = match self.state {
            State::Clearing { since } => 1.0 - ((now - since) / CLEAR_SECS).clamp(0.0, 1.0),
            _ => 0.0,
        };
        for (y, row) in self.well.iter().enumerate() {
            let brightness = if self.clearing.contains(&y) {
                flash
            } else {
                BLOCK_BRIGHTNESS
            };
            for (x, &filled) in row.iter().enumerate() {
                if filled {
                    add_cell(&mut light, x as i32, y as f32, brightness);
                }
            }
        }

        match self.state {
            State::Ready | State::Playing => {
                let landing = self.landing();
                if landing != self.piece {
                    for (x, y) in landing.cells() {
                        add_cell(&mut light, x, y as f32, GHOST_BRIGHTNESS);
                    }
                }
                for (x, y) in self.piece.cells() {
                    add_cell(&mut light, x, y as f32 + self.fall, PIECE_BRIGHTNESS);
                }
            }
            State::Over { since } => {
                if let Some(burst) = &self.burst {
                    burst.draw(&mut light, now - since, OVER_SECS);
                }
            }
            State::Clearing { .. } => {}
        }

        let dim = match self.state {
            State::Over { since } if now - since >= OVER_SECS => 0.4,
            _ => 1.0,
        };
        to_grid(&light, dim)
    }

    fn status(&self, fps: f64) -> String {
        let state = match self.state {
            State::Ready => "Press any key to start",
            State::Over { .. } => "Game over! Space to play again, Q to quit",
            _ => "",
        };
        format!(
            "Score {}  Lines {}  Level {}  Best {}  {:.0} FPS  {}",
            self.score,
            self.lines,
            self.level(),
            self.best,
            fps,
            state
        )
    }
}

/// Play until the player quits, as fast as possible or limited to fps
pub fn ledris_cmd(serialdevs: &[String], extend: bool, fps: Option<u32>) {
    play(
        serialdevs,
        extend,
        fps,
        "Left and right move, up or X turns, Z turns the other way,\n\
         down drops faster and space drops all the way.",
        Game::new,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inputmodule::WIDTH;

    fn sorted(mut cells: [(i32, i32); 4]) -> [(i32, i32); 4] {
        cells.sort();
        cells
    }

    #[test]
    fn four_turns_go_all_the_way_around() {
        for kind in 0..SHAPES.len() {
            let piece = Piece::spawn(kind, WIDTH);
            let mut turned = piece;
            for _ in 0..4 {
                turned.rot += 1;
                // Turning keeps it in its square
                let size = SHAPES[kind].0;
                assert!(turned.cells().iter().all(|&(x, y)| {
                    (piece.x..piece.x + size).contains(&x) && (piece.y..piece.y + size).contains(&y)
                }));
            }
            assert_eq!(sorted(turned.cells()), sorted(piece.cells()));
        }
    }

    #[test]
    fn bag_has_each_block_once() {
        let mut game = Game::new(WIDTH);
        game.bag.clear();
        let mut kinds: Vec<usize> = (0..14).map(|_| game.take_from_bag()).collect();
        kinds[..7].sort();
        kinds[7..].sort();
        assert_eq!(kinds[..7], [0, 1, 2, 3, 4, 5, 6]);
        assert_eq!(kinds[7..], [0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn turning_against_the_wall_kicks_away_from_it() {
        let mut game = Game::new(WIDTH);
        game.state = State::Playing;
        // The long block, standing up against the left wall
        game.piece = Piece {
            kind: 0,
            rot: 1,
            x: -2,
            y: 5,
        };
        assert!(game.fits(game.piece));
        game.rotate(1);
        assert_eq!(game.piece.rot, 2);
        assert!(game.fits(game.piece));
    }

    #[test]
    fn full_rows_disappear_and_the_rest_drops() {
        let mut game = Game::new(WIDTH);
        game.state = State::Playing;
        let bottom = WELL_H - 1;
        // Bottom row full except where the long block goes, something on top
        for x in 0..WIDTH - 4 {
            game.well[bottom][x] = true;
        }
        game.well[bottom - 1][0] = true;
        game.piece = Piece::spawn(0, WIDTH);
        game.piece.x = WIDTH as i32 - 4;
        game.hard_drop(1.0);
        assert_eq!(game.state, State::Clearing { since: 1.0 });
        assert_eq!(game.clearing, [bottom]);
        assert_eq!(game.lines, 1);

        game.update(&mut Input::default(), 1.1 + CLEAR_SECS, CLEAR_SECS);
        assert_eq!(game.state, State::Playing);
        assert!(game.well[bottom][0]);
        assert_eq!(game.well.iter().flatten().filter(|&&c| c).count(), 1);
    }

    #[test]
    fn blocks_lock_after_landing() {
        let mut game = Game::new(WIDTH);
        game.state = State::Playing;
        let mut now = 0.0;
        let dt = 1.0 / 60.0;
        let mut input = Input::default();
        while game.well.iter().flatten().all(|&c| !c) {
            now += dt;
            game.update(&mut input, now, dt);
            assert!(now < WELL_H as f32 / START_GRAVITY + LOCK_DELAY + 1.0);
        }
        assert!(game.well[WELL_H - 1].iter().any(|&c| c));
    }

    /// Height of each column, holes, and how uneven it is, after the block lands at piece
    fn placement_cost(game: &Game, piece: Piece) -> f32 {
        let mut well = game.well.clone();
        for (x, y) in piece.cells() {
            well[y as usize][x as usize] = true;
        }
        let full = well.iter().filter(|row| row.iter().all(|&c| c)).count();
        well.retain(|row| !row.iter().all(|&c| c));
        while well.len() < WELL_H {
            well.insert(0, vec![false; game.width]);
        }
        let heights: Vec<usize> = (0..game.width)
            .map(|x| (0..WELL_H).find(|&y| well[y][x]).map_or(0, |y| WELL_H - y))
            .collect();
        let holes = (0..game.width)
            .map(|x| {
                (WELL_H - heights[x]..WELL_H)
                    .filter(|&y| !well[y][x])
                    .count()
            })
            .sum::<usize>();
        let bumps: usize = heights.windows(2).map(|w| w[0].abs_diff(w[1])).sum();
        let total: usize = heights.iter().sum();
        0.51 * total as f32 + 0.36 * holes as f32 + 0.18 * bumps as f32 - 0.76 * full as f32
    }

    /// A player who tries every place for each block clears plenty of rows,
    /// so full rows are found, scored and removed right, also with --extend
    #[test]
    fn autoplay_clears_rows() {
        for (width, min_lines) in [(WIDTH, 300), (2 * WIDTH, 120)] {
            let mut game = Game::new(width);
            game.state = State::Playing;
            let mut now = 0.0;
            for _ in 0..1500 {
                let start = Piece::spawn(game.piece.kind, width);
                let best = (0..4)
                    .flat_map(|rot| (-3..width as i32).map(move |x| Piece { rot, x, ..start }))
                    .filter(|&p| game.fits(p))
                    .map(|p| {
                        let mut game = game.clone();
                        game.piece = p;
                        let landing = game.landing();
                        (placement_cost(&game, landing), p)
                    })
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .unwrap()
                    .1;
                game.piece = best;
                game.hard_drop(now);
                while game.state != State::Playing {
                    assert!(
                        !matches!(game.state, State::Over { .. }),
                        "Game over after {} rows
{}",
                        game.lines,
                        game.well
                            .iter()
                            .map(|row| row
                                .iter()
                                .map(|&c| if c { '#' } else { '.' })
                                .collect::<String>())
                            .collect::<Vec<_>>()
                            .join(
                                "
"
                            )
                    );
                    now += 0.1;
                    game.update(&mut Input::default(), now, 0.1);
                }
                game.render(now);
            }
            assert!(
                game.lines >= min_lines,
                "Only {} rows at width {}",
                game.lines,
                width
            );
            assert!(game.level() > 0);
            println!("Cleared {} rows, score {}", game.lines, game.score);
        }
    }
}
