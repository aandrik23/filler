use std::collections::VecDeque;

/// Returns true iff placing `piece` at board position (x=col, y=row) is legal:
/// exactly 1 filled piece cell overlaps `my_chars`, 0 overlap `opp_chars`,
/// and the piece fits entirely within the grid bounds.
pub fn is_valid_placement(
    grid: &[Vec<char>],
    piece: &[Vec<char>],
    x: usize,
    y: usize,
    my_chars: &[char; 2],
    opp_chars: &[char; 2],
) -> bool {
    let board_rows = grid.len();
    let board_cols = if board_rows > 0 { grid[0].len() } else { return false };
    let piece_rows = piece.len();
    let piece_cols = if piece_rows > 0 { piece[0].len() } else { return false };

    // Bounds check (usize arithmetic: overflow would wrap, so check before add)
    if x + piece_cols > board_cols || y + piece_rows > board_rows {
        return false;
    }

    let mut my_overlap = 0usize;

    for pr in 0..piece_rows {
        for pc in 0..piece_cols {
            let piece_cell = piece[pr][pc];
            if piece_cell != 'O' && piece_cell != '#' {
                continue;
            }
            let board_cell = grid[y + pr][x + pc];
            if my_chars.contains(&board_cell) {
                my_overlap += 1;
            } else if opp_chars.contains(&board_cell) {
                return false;
            }
        }
    }

    my_overlap == 1
}

/// Multi-source BFS from all `start_positions` simultaneously.
/// Counts reachable '.' cells. Expansion is blocked by any cell containing
/// a char in `blocked_chars`; blocked cells are not counted and not expanded.
#[allow(dead_code)] // used in unit tests (flood_fill_basic)
pub fn flood_fill_count(
    grid: &[Vec<char>],
    start_positions: Vec<(usize, usize)>,
    blocked_chars: &[char],
) -> usize {
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { return 0 };

    let mut visited = vec![vec![false; cols]; rows];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    let mut count = 0usize;

    for (r, c) in start_positions {
        if r < rows && c < cols && !visited[r][c] {
            visited[r][c] = true;
            queue.push_back((r, c));
        }
    }

    while let Some((r, c)) = queue.pop_front() {
        for (dr, dc) in [(-1i64, 0), (1, 0), (0, -1i64), (0, 1)] {
            let nr = r as i64 + dr;
            let nc = c as i64 + dc;
            if nr < 0 || nc < 0 || nr as usize >= rows || nc as usize >= cols {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if visited[nr][nc] {
                continue;
            }
            visited[nr][nc] = true;
            let cell = grid[nr][nc];
            if blocked_chars.contains(&cell) {
                continue; // blocked: do not count, do not expand
            }
            if cell == '.' {
                count += 1;
                queue.push_back((nr, nc));
            }
            // non-'.', non-blocked cells: visited but not counted, not expanded
        }
    }

    count
}
