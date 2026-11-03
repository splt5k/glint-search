/// Pontuação de correspondência: exato > prefixo > início de palavra > sigla > trecho > letras em ordem.
pub fn score(query: &str, target: &str) -> i32 {
    score_lower(&query.trim().to_lowercase(), &target.trim().to_lowercase())
}

/// Versão para strings já em minúsculas (usada no índice, que guarda os nomes em minúsculas).
pub fn score_lower(q: &str, t: &str) -> i32 {
    if q.is_empty() || t.is_empty() {
        return 0;
    }
    if q == t {
        return 1200;
    }
    let qlen = q.chars().count() as i32;
    let tlen = t.chars().count() as i32;

    if t.starts_with(q) {
        return 1000 - (tlen - qlen).clamp(0, 100);
    }

    let words: Vec<&str> = t
        .split(|c: char| c == ' ' || c == '-' || c == '_' || c == '.')
        .filter(|w| !w.is_empty())
        .collect();

    for (i, w) in words.iter().enumerate() {
        if w.starts_with(q) {
            return 850 - (i as i32 * 20);
        }
    }

    if words.len() > 1 && qlen >= 2 && qlen as usize <= words.len() {
        let acronym: String = words.iter().filter_map(|w| w.chars().next()).collect();
        if acronym.starts_with(q) {
            return 800;
        }
        if acronym.contains(q) {
            return 700;
        }
    }

    if let Some(pos) = t.find(q) {
        let chars_before = t[..pos].chars().count() as i32;
        return 600 - (chars_before * 10).min(200);
    }

    let qc: Vec<char> = q.chars().collect();
    let mut qi = 0usize;
    let mut consecutive = 0;
    let mut matched = 0;
    for c in t.chars() {
        if qi < qc.len() && qc[qi] == c {
            matched += 10 + consecutive * 15;
            consecutive += 1;
            qi += 1;
        } else {
            consecutive = 0;
        }
    }
    if qi == qc.len() {
        let penalty = (tlen - qlen) * 2;
        return (400 + matched - penalty).max(100);
    }
    0
}
