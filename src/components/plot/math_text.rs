//! A small LaTeX subset for titles, labels, legends, and annotations.
//!
//! The renderer converts commands and scripts to Unicode text, so the same string is
//! measured, painted, and exported without a math layout engine:
//! `\alpha`, `^{2}`, `_{i}`, `\frac{a}{b}`, `\sqrt{x}`, `\pm`, `\times`, `\cdot`,
//! `\infty`, `\degree`, and units.

/// Convert a LaTeX subset to Unicode text.
///
/// ```
/// # use egui_workbench::components::plot::to_unicode_math;
/// assert_eq!(to_unicode_math(r"$\alpha_{i}^{2}$"), "αᵢ²");
/// assert_eq!(to_unicode_math(r"\frac{V}{s}"), "V/s");
/// ```
pub fn to_unicode_math(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '$' => {}
            '\\' => {
                let mut command = String::new();
                while let Some(next) = chars.peek().copied() {
                    if next.is_ascii_alphabetic() {
                        command.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if command.is_empty() {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                    continue;
                }
                match command.as_str() {
                    "alpha" => out.push('α'),
                    "beta" => out.push('β'),
                    "gamma" => out.push('γ'),
                    "delta" => out.push('δ'),
                    "epsilon" => out.push('ε'),
                    "zeta" => out.push('ζ'),
                    "eta" => out.push('η'),
                    "theta" => out.push('θ'),
                    "iota" => out.push('ι'),
                    "kappa" => out.push('κ'),
                    "lambda" => out.push('λ'),
                    "mu" => out.push('μ'),
                    "nu" => out.push('ν'),
                    "xi" => out.push('ξ'),
                    "pi" => out.push('π'),
                    "rho" => out.push('ρ'),
                    "sigma" => out.push('σ'),
                    "tau" => out.push('τ'),
                    "upsilon" => out.push('υ'),
                    "phi" => out.push('φ'),
                    "chi" => out.push('χ'),
                    "psi" => out.push('ψ'),
                    "omega" => out.push('ω'),
                    "Gamma" => out.push('Γ'),
                    "Delta" => out.push('Δ'),
                    "Theta" => out.push('Θ'),
                    "Lambda" => out.push('Λ'),
                    "Xi" => out.push('Ξ'),
                    "Pi" => out.push('Π'),
                    "Sigma" => out.push('Σ'),
                    "Phi" => out.push('Φ'),
                    "Psi" => out.push('Ψ'),
                    "Omega" => out.push('Ω'),
                    "pm" => out.push('±'),
                    "mp" => out.push('∓'),
                    "times" => out.push('×'),
                    "cdot" => out.push('·'),
                    "infty" => out.push('∞'),
                    "degree" => out.push('°'),
                    "sqrt" => {
                        let group = group(&mut chars);
                        out.push('√');
                        out.push_str(&group);
                    }
                    "frac" => {
                        let numerator = group(&mut chars);
                        let denominator = group(&mut chars);
                        out.push_str(&numerator);
                        out.push('/');
                        out.push_str(&denominator);
                    }
                    "text" | "mathrm" | "operatorname" => out.push_str(&group(&mut chars)),
                    "si" => out.push_str(&group(&mut chars)),
                    _ => out.push_str(&command),
                }
            }
            '^' => out.push_str(&script(&mut chars, true)),
            '_' => out.push_str(&script(&mut chars, false)),
            '{' | '}' => {}
            other => out.push(other),
        }
    }
    out
}

fn group(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    while chars.peek().is_some_and(|character| character.is_whitespace()) {
        chars.next();
    }
    if chars.peek() == Some(&'{') {
        chars.next();
        let mut value = String::new();
        let mut depth = 1;
        for character in chars.by_ref() {
            match character {
                '{' => {
                    depth += 1;
                    value.push(character);
                }
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    value.push(character);
                }
                _ => value.push(character),
            }
        }
        to_unicode_math(&value)
    } else {
        chars.next().map_or_else(String::new, |character| character.to_string())
    }
}

fn script(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, superscript: bool) -> String {
    let raw = group(chars);
    raw.chars()
        .map(|character| {
            let map: &[(char, char)] = if superscript { &SUPERSCRIPT } else { &SUBSCRIPT };
            map.iter()
                .find(|(plain, _)| *plain == character)
                .map_or(character, |(_, mapped)| *mapped)
        })
        .collect()
}

const SUPERSCRIPT: [(char, char); 36] = [
    ('0', '⁰'),
    ('1', '¹'),
    ('2', '²'),
    ('3', '³'),
    ('4', '⁴'),
    ('5', '⁵'),
    ('6', '⁶'),
    ('7', '⁷'),
    ('8', '⁸'),
    ('9', '⁹'),
    ('+', '⁺'),
    ('-', '⁻'),
    ('=', '⁼'),
    ('(', '⁽'),
    (')', '⁾'),
    ('a', 'ᵃ'),
    ('b', 'ᵇ'),
    ('c', 'ᶜ'),
    ('d', 'ᵈ'),
    ('e', 'ᵉ'),
    ('f', 'ᶠ'),
    ('g', 'ᵍ'),
    ('h', 'ʰ'),
    ('i', 'ⁱ'),
    ('j', 'ʲ'),
    ('k', 'ᵏ'),
    ('l', 'ˡ'),
    ('m', 'ᵐ'),
    ('n', 'ⁿ'),
    ('o', 'ᵒ'),
    ('p', 'ᵖ'),
    ('r', 'ʳ'),
    ('s', 'ˢ'),
    ('t', 'ᵗ'),
    ('u', 'ᵘ'),
    ('v', 'ᵛ'),
];
const SUBSCRIPT: [(char, char); 26] = [
    ('0', '₀'),
    ('1', '₁'),
    ('2', '₂'),
    ('3', '₃'),
    ('4', '₄'),
    ('5', '₅'),
    ('6', '₆'),
    ('7', '₇'),
    ('8', '₈'),
    ('9', '₉'),
    ('+', '₊'),
    ('-', '₋'),
    ('=', '₌'),
    ('(', '₍'),
    (')', '₎'),
    ('a', 'ₐ'),
    ('e', 'ₑ'),
    ('h', 'ₕ'),
    ('i', 'ᵢ'),
    ('j', 'ⱼ'),
    ('k', 'ₖ'),
    ('l', 'ₗ'),
    ('m', 'ₘ'),
    ('n', 'ₙ'),
    ('o', 'ₒ'),
    ('p', 'ₚ'),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greek_scripts_and_fractions() {
        assert_eq!(to_unicode_math(r"$\alpha_{i}^{2}$"), "αᵢ²");
        assert_eq!(to_unicode_math(r"\frac{V}{s}"), "V/s");
        assert_eq!(to_unicode_math(r"\sqrt{x}"), "√x");
        assert_eq!(to_unicode_math(r"25\pm3\degree"), "25±3°");
        assert_eq!(to_unicode_math(r"a \cdot b \times c"), "a · b × c");
    }
}
