pub fn read_pgns(file_path: &str) -> Vec<Vec<String>> {
    eprintln!("INFO: reading from {file_path}");

    let raw_pgns = std::fs::read_to_string(file_path).expect(&format!(
        "ERROR: should be able to read from: {}",
        file_path
    ));

    let split_pgns = split_pgns(&raw_pgns);
    eprintln!("INFO: got {} notations", split_pgns.len());

    let move_sequences = split_pgns
        .iter()
        .map(|notation| move_sequence(notation))
        .collect();

    move_sequences
}

/// split notation into individual moves
/// result aka 1-0 | 0-1 | 1/2-1/2 is not included
fn move_sequence(notation: &str) -> Vec<String> {
    let last_space = notation
        .rfind(" ")
        .expect("PGN is quaranteed to have space");

    // PERF: consider optimization
    notation
        .split_at(last_space)
        .0
        .split_whitespace()
        .map(|ln| {
            if !ln.contains(".") {
                return ln;
            }
            let dot = ln.find(".").expect("should contain dot");
            ln.split_at(dot + 1).1
        })
        .map(|ln| ln.trim())
        .map(String::from)
        .collect()
}

/// removes tags from pgn/s
fn strip_metadata(pgn: &str) -> Vec<String> {
    pgn.lines()
        .filter(|ln| !ln.is_empty())
        .filter(|ln| !ln.starts_with("["))
        .map(String::from)
        .collect()
}

/// splits pgns across Vector
/// if provided &str contains only one pgn Vector's size is 1
fn split_pgns(pgns: &str) -> Vec<String> {
    let formatted_pgns = strip_metadata(pgns);
    let mut pgn_buff = Vec::new();
    let mut str_buff = String::new();

    for ln in formatted_pgns.iter() {
        if !(ln.contains("1-0") || ln.contains("0-1") || ln.contains("1/2")) {
            str_buff.push_str(ln);
        } else {
            str_buff.push_str(ln);
            pgn_buff.push(str_buff.clone());
            str_buff.clear();
        }
    }

    pgn_buff
}
