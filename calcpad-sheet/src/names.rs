//! Names: Greek spellings become Greek letters and `star` becomes a subscript, so that
//! `phi`, `sigma_c` and `Mstar` typeset as phi, sigma with subscript c and M with subscript star.

/// (spelling, lower case letter, upper case letter). The upper case letter is only used when it
/// differs from a Latin capital, so `Mu` and `Eta` stay Latin.
const GREEK: &[(&str, char, Option<char>)] = &[
    ("alpha", 'α', None),
    ("beta", 'β', None),
    ("gamma", 'γ', Some('Γ')),
    ("delta", 'δ', Some('Δ')),
    ("epsilon", 'ε', None),
    ("zeta", 'ζ', None),
    ("eta", 'η', None),
    ("theta", 'θ', Some('Θ')),
    ("iota", 'ι', None),
    ("kappa", 'κ', None),
    ("lambda", 'λ', Some('Λ')),
    ("mu", 'μ', None),
    ("nu", 'ν', None),
    ("xi", 'ξ', Some('Ξ')),
    ("omicron", 'ο', None),
    ("pi", 'π', Some('Π')),
    ("rho", 'ρ', None),
    ("sigma", 'σ', Some('Σ')),
    ("tau", 'τ', None),
    ("upsilon", 'υ', Some('Υ')),
    ("phi", 'φ', Some('Φ')),
    ("chi", 'χ', None),
    ("psi", 'ψ', Some('Ψ')),
    ("omega", 'ω', Some('Ω')),
];

/// True for a character that may be part of a name.
pub fn is_name_char(c: char) -> bool {
    (c.is_alphanumeric() && !matches!(c, '⁰'..='⁹' | '²' | '³' | '¹')) || c == '_' || c == '′' || c == '″'
}

/// True for a character a name may start with.
pub fn is_name_start(c: char) -> bool {
    c.is_alphabetic()
}

/// True if `s` is a plain name (no function arguments).
pub fn is_name(s: &str) -> bool {
    let mut it = s.chars();
    matches!(it.next(), Some(c) if is_name_start(c)) && it.all(is_name_char)
}

/// The Greek letter of a spelling that is the whole segment: `phi`, `Delta`.
fn whole(seg: &str) -> Option<char> {
    for (name, lower, upper) in GREEK {
        if seg == *name {
            return Some(*lower);
        }
        if let Some(u) = upper {
            let mut cs = seg.chars();
            if let Some(f) = cs.next() {
                if f.is_ascii_uppercase() && f.to_ascii_lowercase().to_string() + cs.as_str() == *name {
                    return Some(*u);
                }
            }
        }
    }
    None
}

/// A Greek spelling at the start of the segment followed by an upper case letter or a digit:
/// `phiMs` is the letter phi then `Ms`.
fn prefixed(seg: &str) -> Option<String> {
    let mut best: Option<(usize, char)> = None;
    for (name, lower, upper) in GREEK {
        let mut candidates = vec![(name.to_string(), *lower)];
        if let Some(u) = upper {
            let mut cap = name.to_string();
            cap[..1].make_ascii_uppercase();
            candidates.push((cap, *u));
        }
        for (spell, letter) in candidates {
            if let Some(rest) = seg.strip_prefix(spell.as_str()) {
                if rest
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                    && best.is_none_or(|(l, _)| spell.len() > l)
                {
                    best = Some((spell.len(), letter));
                }
            }
        }
    }
    best.map(|(l, letter)| format!("{letter}{}", &seg[l..]))
}

fn segment(seg: &str) -> String {
    if let Some(g) = whole(seg) {
        return g.to_string();
    }
    if let Some(base) = seg.strip_suffix("star") {
        if !base.is_empty()
            && base
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        {
            return format!("{}_star", prefixed(base).unwrap_or_else(|| base.to_string()));
        }
    }
    prefixed(seg).unwrap_or_else(|| seg.to_string())
}

/// The Calcpad spelling of a sheet name.
pub fn rename(name: &str) -> String {
    name.split('_').map(segment).collect::<Vec<_>>().join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greek_and_subscripts() {
        assert_eq!(rename("phi"), "φ");
        assert_eq!(rename("sigma_c"), "σ_c");
        assert_eq!(rename("Delta_T"), "Δ_T");
        assert_eq!(rename("phiMs"), "φMs");
        assert_eq!(rename("Mstar"), "M_star");
        assert_eq!(rename("Mu"), "Mu");
        assert_eq!(rename("fy"), "fy");
        assert_eq!(rename("pitch"), "pitch");
        assert_eq!(rename("lambda_1"), "λ_1");
        assert_eq!(rename("sigma2"), "σ2");
    }
}
