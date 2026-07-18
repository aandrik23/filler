pub struct GameState {
    #[allow(dead_code)]
    pub player_num: u8,
    pub my_chars: [char; 2],
    pub opp_chars: [char; 2],
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<Vec<char>>,
    pub piece: Vec<Vec<char>>,
    pub piece_rows: usize,
    pub piece_cols: usize,
    #[allow(dead_code)]
    pub turn: u32,
}

pub fn parse_player_number(line: &str) -> u8 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 3 && parts[2] == "p1" { 1 } else { 2 }
}

pub fn parse_anfield_header(line: &str) -> (usize, usize) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let cols: usize = parts[1].parse().expect("invalid anfield cols");
    let rows: usize = parts[2].trim_end_matches(':').parse().expect("invalid anfield rows");
    (cols, rows)
}

pub fn parse_grid_row(line: &str, cols: usize) -> Vec<char> {
    let content = match line.find(' ') {
        Some(idx) => &line[idx + 1..],
        None => "",
    };
    let mut row: Vec<char> = content.trim_end().chars().take(cols).collect();
    while row.len() < cols {
        row.push('.');
    }
    row
}

pub fn parse_piece_header(line: &str) -> (usize, usize) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let cols: usize = parts[1].parse().expect("invalid piece cols");
    let rows: usize = parts[2].trim_end_matches(':').parse().expect("invalid piece rows");
    (cols, rows)
}

pub fn parse_piece_row(line: &str) -> Vec<char> {
    line.trim_end().chars().collect()
}
