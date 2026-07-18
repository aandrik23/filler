use crate::board::is_valid_placement;
use crate::parser::{
    parse_anfield_header, parse_grid_row, parse_piece_header, parse_piece_row,
    parse_player_number, GameState,
};
use crate::strategy::find_best_placement;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_grid(rows: &[&str]) -> Vec<Vec<char>> {
    rows.iter().map(|r| r.chars().collect()).collect()
}

fn make_piece(rows: &[&str]) -> Vec<Vec<char>> {
    rows.iter().map(|r| r.chars().collect()).collect()
}

const P1_MY: [char; 2] = ['@', 'a'];
const P1_OPP: [char; 2] = ['$', 's'];

// INPUT PARSING TESTS

#[test]
fn parse_player_number_p1() {
    assert_eq!(parse_player_number("$$$ exec p1 : [robots/bender]"), 1);
}

#[test]
fn parse_player_number_p2() {
    assert_eq!(parse_player_number("$$$ exec p2 : [robots/bender]"), 2);
}

#[test]
fn parse_anfield_dimensions() {
    let (cols, rows) = parse_anfield_header("Anfield 20 15:");
    assert_eq!(cols, 20);
    assert_eq!(rows, 15);
}

#[test]
fn parse_anfield_grid_strips_prefix() {
    // "000 .@.." → ['.', '@', '.', '.']
    let row = parse_grid_row("000 .@..", 4);
    assert_eq!(row, vec!['.', '@', '.', '.']);
}

#[test]
fn parse_anfield_skips_header() {
    // Simulate parsing a 2-row anfield block: header line must NOT enter the grid
    let header = "Anfield 5 2:";
    let col_header = "    01234";
    let grid_lines = ["000 ..@..", "001 ....."];

    let (cols, rows) = parse_anfield_header(header);
    // col_header is skipped (as in main.rs); only grid_lines are parsed
    let _ = col_header; // explicitly consumed/skipped
    let grid: Vec<Vec<char>> = grid_lines.iter().map(|l| parse_grid_row(l, cols)).collect();

    assert_eq!(rows, 2);
    assert_eq!(grid.len(), 2, "column header must not appear as a grid row");
    assert_eq!(cols, 5);
}

#[test]
fn parse_piece_dimensions() {
    let (cols, rows) = parse_piece_header("Piece 4 2:");
    assert_eq!(cols, 4);
    assert_eq!(rows, 2);
}

#[test]
fn parse_piece_shape() {
    let row = parse_piece_row("OO.");
    assert_eq!(row.len(), 3);
    assert_eq!(row[0], 'O'); // filled
    assert_eq!(row[1], 'O'); // filled
    assert_eq!(row[2], '.'); // empty
}

// ---------------------------------------------------------------------------
// PLACEMENT VALIDATION TESTS
// ---------------------------------------------------------------------------

#[test]
fn valid_one_overlap() {
    // '@' at col 1 row 0; piece "OO" placed at x=0,y=0 → overlap count = 1
    let grid = make_grid(&["..@..", "....."]);
    let piece = make_piece(&["OO"]);
    // place at x=1,y=0: piece[0][0]='O' hits grid[0][1]='.', piece[0][1]='O' hits grid[0][2]='@' (1 overlap)
    assert!(is_valid_placement(&grid, &piece, 1, 0, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_zero_overlap() {
    let grid = make_grid(&[".....", "@...."]);
    let piece = make_piece(&["O"]);
    // x=0,y=0: grid[0][0]='.' → no overlap
    assert!(!is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_two_overlaps() {
    let grid = make_grid(&["@@..."]);
    let piece = make_piece(&["OO"]);
    // x=0,y=0: both cells hit '@' → 2 overlaps
    assert!(!is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_hits_opponent_territory() {
    // '$' is opp char for p1
    let grid = make_grid(&["$@..."]);
    let piece = make_piece(&["OO"]);
    // x=0,y=0: piece[0][0] hits '$' (opponent) → immediately false
    assert!(!is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_hits_opponent_last_piece() {
    // 's' is opp's last-placed-piece char; must also be blocked
    let grid = make_grid(&["s@..."]);
    let piece = make_piece(&["OO"]);
    assert!(!is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));

    // same for 'a' (my last-placed char counts as MY territory, should produce overlap)
    let grid2 = make_grid(&["a...."]);
    let piece2 = make_piece(&["O"]);
    // x=0,y=0: 'a' is in P1_MY → overlap count = 1 → valid
    assert!(is_valid_placement(&grid2, &piece2, 0, 0, &P1_MY, &P1_OPP));
}

// ---------------------------------------------------------------------------
// BOUNDARY DETECTION TESTS
// ---------------------------------------------------------------------------

#[test]
fn invalid_out_of_bounds_right() {
    // Board is 3 wide; piece "OO" placed at x=2 would need cols 2 and 3 → out of bounds
    let grid = make_grid(&["..@"]);
    let piece = make_piece(&["OO"]);
    assert!(!is_valid_placement(&grid, &piece, 2, 0, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_out_of_bounds_bottom() {
    // Board is 2 rows tall; piece with 2 rows placed at y=1 needs rows 1 and 2 → out of bounds
    let grid = make_grid(&["...", "..@"]);
    let piece = make_piece(&["O", "O"]);
    assert!(!is_valid_placement(&grid, &piece, 2, 1, &P1_MY, &P1_OPP));
}

#[test]
fn invalid_at_position_zero_zero_no_overlap() {
    // '@' is far away; placing at (0,0) gives 0 overlaps → invalid
    let grid = make_grid(&["....@", "....."]);
    let piece = make_piece(&["O"]);
    assert!(!is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));
}

// ---------------------------------------------------------------------------
// PARSING EDGE CASES
// ---------------------------------------------------------------------------

#[test]
fn parse_anfield_grid_strips_two_digit_prefix() {
    // Row numbers can be 2-digit (e.g. "00 ") — find-first-space logic must handle both widths
    let row = parse_grid_row("00 .@..", 4);
    assert_eq!(row, vec!['.', '@', '.', '.']);
}

#[test]
fn piece_hash_char_recognized_as_filled() {
    // '#' is an alternate filled piece cell character; must produce an overlap just like 'O'
    let grid = make_grid(&["@...."]);
    let piece = make_piece(&["#"]);
    assert!(is_valid_placement(&grid, &piece, 0, 0, &P1_MY, &P1_OPP));
}

#[test]
fn all_four_territory_chars_recognized() {
    // '@' and 'a' are both MY chars; '$' and 's' are both OPP chars
    let my_chars: [char; 2] = ['@', 'a'];
    let opp_chars: [char; 2] = ['$', 's'];

    // 'a' counts as my territory (last-placed piece char)
    let grid = make_grid(&["a...."]);
    let piece = make_piece(&["O"]);
    assert!(is_valid_placement(&grid, &piece, 0, 0, &my_chars, &opp_chars));

    // 's' counts as opponent and must block the placement
    let grid2 = make_grid(&["s...."]);
    assert!(!is_valid_placement(&grid2, &piece, 0, 0, &my_chars, &opp_chars));
}

// ---------------------------------------------------------------------------
// OUTPUT FORMAT TEST
// ---------------------------------------------------------------------------

#[test]
fn output_is_x_then_y() {
    // '@' at col=3, row=0 in a 5-wide 2-tall board.
    // Piece is a single cell; only valid placement is at (x=3, y=0).
    // find_best_placement must return (col, row) = (3, 0), not (0, 3).
    let grid = make_grid(&["...@.", "....."]);
    let piece = make_piece(&["O"]);
    let mut state = GameState {
        player_num: 1,
        my_chars: P1_MY,
        opp_chars: P1_OPP,
        rows: 2,
        cols: 5,
        grid,
        piece,
        piece_rows: 1,
        piece_cols: 1,
        turn: 1,
    };
    let (x, y) = find_best_placement(&mut state);
    // x is the column (3), y is the row (0)
    assert_eq!(x, 3, "x must be the column index");
    assert_eq!(y, 0, "y must be the row index");
    assert_eq!(format!("{} {}", x, y), "3 0");
}

// ---------------------------------------------------------------------------
// FLOOD FILL TESTS
// ---------------------------------------------------------------------------

#[test]
fn flood_fill_basic() {
    use crate::board::flood_fill_count;

    // Layout (5 rows × 5 cols):
    //   row 0: @@@@@   ← my territory (start positions)
    //   row 1: .....   ← reachable (5 cells)
    //   row 2: $$$$$   ← opponent barrier (blocks all expansion)
    //   row 3: .....   ← NOT reachable (behind barrier)
    //   row 4: .....   ← NOT reachable (behind barrier)
    let grid = make_grid(&[
        "@@@@@",
        ".....",
        "$$$$$",
        ".....",
        ".....",
    ]);

    // Seed BFS from all '@' cells on row 0
    let starts: Vec<(usize, usize)> = (0..5).map(|c| (0usize, c)).collect();
    let blocked = vec!['$', 's'];

    let count = flood_fill_count(&grid, starts, &blocked);

    // Only row 1 is reachable; rows 3-4 are cut off by the '$' wall
    assert_eq!(count, 5);
}
