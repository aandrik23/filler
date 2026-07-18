mod board;
mod parser;
mod strategy;

#[cfg(test)]
mod tests;

use std::io::{self, BufRead, Write};

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut line = String::new();

    // Drain ALL "$$$ exec" lines. The first one tells us our player number.
    // The first non-exec, non-blank line is the Anfield header — save it for
    // the game loop so we don't lose it.
    let mut player_num = 2u8;
    let mut exec_count = 0usize;

    let first_game_line: String = loop {
        line.clear();
        let n = match reader.read_line(&mut line) {
            Ok(n) => n,
            Err(_) => return,
        };
        if n == 0 {
            return;
        }
        let trimmed = line.trim();
        if trimmed.starts_with("$$$") && trimmed.contains("exec") {
            if exec_count == 0 {
                player_num = parser::parse_player_number(trimmed);
            }
            exec_count += 1;
        } else if !trimmed.is_empty() {
            break line.clone();
        }
    };

    let (my_chars, opp_chars): ([char; 2], [char; 2]) = if player_num == 1 {
        (['@', 'a'], ['$', 's'])
    } else {
        (['$', 's'], ['@', 'a'])
    };

    let mut prefetch: Option<String> = Some(first_game_line);
    let mut turn = 0u32;

    loop {
        turn += 1;

        // ---- locate Anfield header ----
        let (cols, rows) = loop {
            let current = match prefetch.take() {
                Some(saved) => saved,
                None => {
                    line.clear();
                    let n = match reader.read_line(&mut line) {
                        Ok(n) => n,
                        Err(_) => return,
                    };
                    if n == 0 {
                        return;
                    }
                    line.clone()
                }
            };
            if current.trim().starts_with("Anfield") {
                break parser::parse_anfield_header(current.trim());
            }
        };

        // ---- skip the column-index header line ----
        line.clear();
        if reader.read_line(&mut line).is_err() {
            return;
        }

        // ---- read `rows` grid lines ----
        let mut grid: Vec<Vec<char>> = Vec::with_capacity(rows);
        for _ in 0..rows {
            line.clear();
            if reader.read_line(&mut line).is_err() {
                return;
            }
            grid.push(parser::parse_grid_row(&line, cols));
        }

        // ---- locate Piece header ----
        let (piece_cols, piece_rows) = loop {
            line.clear();
            let n = match reader.read_line(&mut line) {
                Ok(n) => n,
                Err(_) => return,
            };
            if n == 0 {
                return;
            }
            if line.trim().starts_with("Piece") {
                break parser::parse_piece_header(line.trim());
            }
        };

        // ---- read `piece_rows` piece lines ----
        let mut piece: Vec<Vec<char>> = Vec::with_capacity(piece_rows);
        for _ in 0..piece_rows {
            line.clear();
            if reader.read_line(&mut line).is_err() {
                return;
            }
            piece.push(parser::parse_piece_row(&line));
        }

        // ---- compute and emit placement ----
        let mut state = parser::GameState {
            player_num,
            my_chars,
            opp_chars,
            rows,
            cols,
            grid,
            piece,
            piece_rows,
            piece_cols,
            turn,
        };

        let (x, y) = strategy::find_best_placement(&mut state);

        println!("{x} {y}");
        if let Err(e) = io::stdout().flush() {
            eprintln!("flush error: {e}");
            return;
        }
    }
}
