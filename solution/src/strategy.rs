use std::collections::VecDeque;
use std::time::Instant;

use crate::board::is_valid_placement;
use crate::parser::GameState;

const TIMEOUT_SECS: u64 = 5;

/// Returns (x=col, y=row) of the best placement, or (0, 0) if none exists.
///
/// Algorithm:
///   1. Collect all valid placements.
///   2. For each placement: clone the grid, apply the piece, compute Voronoi
///      territorial score (cells reachable by me first minus cells reachable by
///      opponent first).  Greedy heuristics break ties.
///   3. Return the highest-scoring placement.
///
/// Voronoi scoring is strictly stronger than binary flood-fill reachability:
/// it answers "who gets there first?" rather than "can I get there at all?",
/// making it much better at identifying blocking/cutting moves.
pub fn find_best_placement(state: &mut GameState) -> (usize, usize) {
    let start = Instant::now();

    let mut candidates: Vec<(usize, usize, usize)> = Vec::new();
    'collect: for y in 0..state.rows {
        for x in 0..state.cols {
            if start.elapsed().as_secs() >= TIMEOUT_SECS {
                break 'collect;
            }
            if is_valid_placement(
                &state.grid,
                &state.piece,
                x,
                y,
                &state.my_chars,
                &state.opp_chars,
            ) {
                let cells = count_empty_covered(state, x, y);
                candidates.push((x, y, cells));
            }
        }
    }

    if candidates.is_empty() {
        return (0, 0);
    }
    if candidates.len() == 1 {
        return (candidates[0].0, candidates[0].1);
    }

    let opp_cells = collect_opp_cells(state);
    let mut best_score = i64::MIN;
    let mut best_pos = (candidates[0].0, candidates[0].1);

    for &(x, y, cells) in &candidates {
        if start.elapsed().as_secs() >= TIMEOUT_SECS {
            break;
        }

        let mut g = state.grid.clone();
        for pr in 0..state.piece_rows {
            for pc in 0..state.piece_cols {
                let cell = state.piece[pr][pc];
                if cell == 'O' || cell == '#' {
                    g[y + pr][x + pc] = state.my_chars[1];
                }
            }
        }

        let voro = voronoi_diff(&g, state.rows, state.cols, &state.my_chars, &state.opp_chars);
        // Greedy tiebreakers: expand cells, stay close to opponent
        let opp_dist = min_opp_dist(state, x, y, &opp_cells);
        let score = voro * 10 + cells as i64 * 3 - opp_dist as i64;

        if score > best_score {
            best_score = score;
            best_pos = (x, y);
        }
    }

    best_pos
}

// ── Voronoi ───────────────────────────────────────────────────────────────

/// Returns (cells reachable by `my_chars` first) − (cells reachable by
/// `opp_chars` first) over all empty '.' cells in the grid.
///
/// Each side does a multi-source BFS from its territory.  For each empty cell
/// the side that arrives with a shorter BFS distance "owns" it; ties are
/// split evenly (counted as 0 for both).
fn voronoi_diff(
    grid: &[Vec<char>],
    rows: usize,
    cols: usize,
    my_chars: &[char; 2],
    opp_chars: &[char; 2],
) -> i64 {
    let my_dist = bfs_distance(grid, rows, cols, my_chars, opp_chars);
    let opp_dist = bfs_distance(grid, rows, cols, opp_chars, my_chars);

    let mut score = 0i64;
    for r in 0..rows {
        for c in 0..cols {
            if grid[r][c] != '.' {
                continue;
            }
            match my_dist[r * cols + c].cmp(&opp_dist[r * cols + c]) {
                std::cmp::Ordering::Less => score += 1,
                std::cmp::Ordering::Greater => score -= 1,
                std::cmp::Ordering::Equal => {}
            }
        }
    }
    score
}

/// Multi-source BFS from all `start_chars` territory cells.
fn bfs_distance(
    grid: &[Vec<char>],
    rows: usize,
    cols: usize,
    start_chars: &[char; 2],
    blocked_chars: &[char; 2],
) -> Vec<u32> {
    const INF: u32 = u32::MAX;
    let mut dist = vec![INF; rows * cols];
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();

    for r in 0..rows {
        for c in 0..cols {
            if start_chars.contains(&grid[r][c]) {
                dist[r * cols + c] = 0;
                queue.push_back((r, c));
            }
        }
    }

    while let Some((r, c)) = queue.pop_front() {
        let d = dist[r * cols + c];
        for (dr, dc) in [(-1i64, 0), (1, 0), (0, -1i64), (0, 1)] {
            let nr = r as i64 + dr;
            let nc = c as i64 + dc;
            if nr < 0 || nc < 0 || nr as usize >= rows || nc as usize >= cols {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            let idx = nr * cols + nc;
            if dist[idx] != INF {
                continue;
            }
            if blocked_chars.contains(&grid[nr][nc]) {
                dist[idx] = INF - 1;
                continue;
            }
            dist[idx] = d + 1;
            queue.push_back((nr, nc));
        }
    }

    dist
}

fn count_empty_covered(state: &GameState, x: usize, y: usize) -> usize {
    let mut count = 0;
    for pr in 0..state.piece_rows {
        for pc in 0..state.piece_cols {
            let cell = state.piece[pr][pc];
            if (cell == 'O' || cell == '#') && state.grid[y + pr][x + pc] == '.' {
                count += 1;
            }
        }
    }
    count
}

fn collect_opp_cells(state: &GameState) -> Vec<(usize, usize)> {
    (0..state.rows)
        .flat_map(|r| {
            (0..state.cols).filter_map(move |c| {
                if state.opp_chars.contains(&state.grid[r][c]) {
                    Some((r, c))
                } else {
                    None
                }
            })
        })
        .collect()
}

fn min_opp_dist(state: &GameState, x: usize, y: usize, opp_cells: &[(usize, usize)]) -> usize {
    if opp_cells.is_empty() {
        return 0;
    }
    let pr = y + state.piece_rows / 2;
    let pc = x + state.piece_cols / 2;
    opp_cells
        .iter()
        .map(|&(r, c)| pr.abs_diff(r) + pc.abs_diff(c))
        .min()
        .unwrap_or(0)
}
