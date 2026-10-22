//! The mapping tables, all in one place: units, built-in functions, constants and the names
//! that mark constructs the importer does not translate.

/// Normalises look-alike characters in a name: the micro sign (U+00B5) becomes Greek mu (U+03BC).
pub fn normalise_name(s: &str) -> String {
    s.replace('\u{00B5}', "\u{03BC}")
}

/// Units whose source spelling is also the Calcpad spelling.
pub const SAME_UNITS: &[&str] = &[
    // length
    "m", "km", "dm", "cm", "mm", "μm", "nm", "pm", "in", "ft", "yd", "mi", "nmi", "th", "ly", "AU", // mass
    "g", "kg", "mg", "μg", "lb", "oz", "slug", "gr", "t", "kt", // time and frequency
    "s", "ms", "μs", "ns", "min", "h", "d", "w", "y", "Hz", "kHz", "MHz", "GHz", "rpm", // electrical
    "A", "mA", "μA", "kA", "V", "mV", "kV", "Ω", "kΩ", "F", "μF", "H", "mH", "C", "S", "T", "Wb",
    // thermodynamic temperature, amount, light
    "K", "°C", "°F", "°R", "mol", "cd", "lm", "lx", // force
    "N", "kN", "MN", "GN", "daN", "kNm", "Nm", "lbf", "kgf", "kip", "dyn", // pressure
    "Pa", "kPa", "MPa", "GPa", "hPa", "μPa", "bar", "mbar", "atm", "psi", "ksi", "psf", "ksf", "mmHg", "inHg", "Torr",
    // energy and power
    "J", "kJ", "MJ", "GJ", "mJ", "cal", "kcal", "BTU", "erg", "eV", "Wh", "kWh", "MWh", "W", "kW", "MW", "GW", "mW",
    "hp", // volume and area
    "L", "mL", "μL", "ha", // angle
    "rad", "deg", "°", "grad", "rev", "%", // speed
    "mph", "knot",
];

/// Units whose source spelling differs from the Calcpad spelling: (source, Calcpad).
pub const RENAMED_UNITS: &[(&str, &str)] = &[
    ("sec", "s"),
    ("hr", "h"),
    ("hour", "h"),
    ("day", "d"),
    ("yr", "y"),
    ("gm", "g"),
    ("tonne", "t"),
    ("lbm", "lb"),
    ("liter", "L"),
    ("litre", "L"),
    ("ml", "mL"),
    ("gal", "gal_US"),
    ("qt", "qt_US"),
    ("pt", "pt_US"),
    ("mile", "mi"),
    ("torr", "Torr"),
    ("dyne", "dyn"),
    ("ohm", "Ω"),
    ("tonf", "tonf_US"),
    ("ton", "ton_US"),
    ("mil", "th"),
    ("acre", "ac"),
    ("nmile", "nmi"),
    ("degC", "°C"),
    ("degF", "°F"),
    ("degR", "°R"),
    ("International_BTU", "BTU"),
    ("pound", "lb"),
    ("foot", "ft"),
    ("inch", "in"),
    ("pounds_per_sq_inch", "psi"),
    ("meter", "m"),
    ("metre", "m"),
    ("gram", "g"),
    ("kilogram", "kg"),
    ("second", "s"),
    ("minute", "min"),
    ("newton", "N"),
    ("pascal", "Pa"),
    ("joule", "J"),
    ("watt", "W"),
    ("volt", "V"),
    ("ampere", "A"),
    ("kelvin", "K"),
    ("hertz", "Hz"),
    ("coulomb", "C"),
    ("farad", "F"),
    ("henry", "H"),
    ("tesla", "T"),
    ("weber", "Wb"),
    ("siemens", "S"),
    ("radian", "rad"),
    ("percent", "%"),
];

/// Units the source language knows and Calcpad does not: reported as `unmapped-unit`.
pub const UNMAPPED_UNITS: &[&str] = &[
    "kmol", "mmol", "lbmol", "gmol", "kgmol", "angstrom", "gal_imp", "dB", "Bq", "Gy", "Sv",
];

/// The Calcpad spelling of a unit name, if the table has one.
pub fn map_unit(name: &str) -> Option<&'static str> {
    let n = normalise_name(name);
    if let Some(u) = SAME_UNITS.iter().find(|u| **u == n) {
        return Some(u);
    }
    RENAMED_UNITS.iter().find(|(s, _)| *s == n).map(|(_, c)| *c)
}

pub fn is_unmapped_unit(name: &str) -> bool {
    UNMAPPED_UNITS.contains(&normalise_name(name).as_str())
}

/// What to do with a call to a built-in function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FnRule {
    /// Same arguments, new name.
    Direct(&'static str),
    /// Direct, but the function depends on the angle mode.
    Trig(&'static str),
    /// `log(x)` or `log(x, base)`.
    Log,
    /// `round(x)` or `round(x, digits)`.
    Round,
}

/// (source name, rule, minimum arguments, maximum arguments)
pub const FUNCTIONS: &[(&str, FnRule, usize, usize)] = &[
    ("sin", FnRule::Trig("sin"), 1, 1),
    ("cos", FnRule::Trig("cos"), 1, 1),
    ("tan", FnRule::Trig("tan"), 1, 1),
    ("sec", FnRule::Trig("sec"), 1, 1),
    ("csc", FnRule::Trig("csc"), 1, 1),
    ("cot", FnRule::Trig("cot"), 1, 1),
    ("asin", FnRule::Trig("asin"), 1, 1),
    ("acos", FnRule::Trig("acos"), 1, 1),
    ("atan", FnRule::Trig("atan"), 1, 1),
    ("atan2", FnRule::Trig("atan2"), 2, 2),
    ("acot", FnRule::Trig("acot"), 1, 1),
    ("asec", FnRule::Trig("asec"), 1, 1),
    ("acsc", FnRule::Trig("acsc"), 1, 1),
    ("sinh", FnRule::Direct("sinh"), 1, 1),
    ("cosh", FnRule::Direct("cosh"), 1, 1),
    ("tanh", FnRule::Direct("tanh"), 1, 1),
    ("sech", FnRule::Direct("sech"), 1, 1),
    ("csch", FnRule::Direct("csch"), 1, 1),
    ("coth", FnRule::Direct("coth"), 1, 1),
    ("asinh", FnRule::Direct("asinh"), 1, 1),
    ("acosh", FnRule::Direct("acosh"), 1, 1),
    ("atanh", FnRule::Direct("atanh"), 1, 1),
    ("ln", FnRule::Direct("ln"), 1, 1),
    ("log", FnRule::Log, 1, 2),
    ("exp", FnRule::Direct("exp"), 1, 1),
    ("sqrt", FnRule::Direct("sqrt"), 1, 1),
    ("abs", FnRule::Direct("abs"), 1, 1),
    ("sign", FnRule::Direct("sign"), 1, 1),
    ("signum", FnRule::Direct("sign"), 1, 1),
    ("floor", FnRule::Direct("floor"), 1, 1),
    ("ceil", FnRule::Direct("ceiling"), 1, 1),
    ("trunc", FnRule::Direct("trunc"), 1, 1),
    ("round", FnRule::Round, 1, 2),
    ("mod", FnRule::Direct("mod"), 2, 2),
    ("min", FnRule::Direct("min"), 1, 64),
    ("max", FnRule::Direct("max"), 1, 64),
    ("gcd", FnRule::Direct("gcd"), 2, 64),
    ("lcm", FnRule::Direct("lcm"), 2, 64),
    ("Re", FnRule::Direct("re"), 1, 1),
    ("Im", FnRule::Direct("im"), 1, 1),
    ("arg", FnRule::Direct("phase"), 1, 1),
    ("if", FnRule::Direct("if"), 3, 3),
    ("rnd", FnRule::Direct("random"), 1, 1),
    ("rows", FnRule::Direct("n_rows"), 1, 1),
    ("cols", FnRule::Direct("n_cols"), 1, 1),
    ("length", FnRule::Direct("len"), 1, 1),
    ("identity", FnRule::Direct("identity"), 1, 1),
    ("matrix", FnRule::Direct("matrix"), 2, 2),
    ("det", FnRule::Direct("det"), 1, 1),
    ("rank", FnRule::Direct("rank"), 1, 1),
    ("augment", FnRule::Direct("augment"), 2, 2),
    ("stack", FnRule::Direct("stack"), 2, 2),
    ("sort", FnRule::Direct("sort"), 1, 1),
    ("reverse", FnRule::Direct("reverse"), 1, 1),
    ("mean", FnRule::Direct("mean"), 1, 64),
    ("norm2", FnRule::Direct("norm_2"), 1, 1),
];

pub fn lookup_function(name: &str) -> Option<(FnRule, usize, usize)> {
    FUNCTIONS
        .iter()
        .find(|(n, ..)| *n == name)
        .map(|(_, r, lo, hi)| (*r, *lo, *hi))
}

/// Built-in functions of the source language that have no Calcpad equivalent in the table.
/// A call to one of these is reported as `unmapped-function`; a call to a name in neither list is
/// `undefined-function` (probably an add-in or an included worksheet).
pub const UNMAPPED_FUNCTIONS: &[&str] = &[
    "stdev",
    "Stdev",
    "var",
    "Var",
    "median",
    "mode",
    "lookup",
    "vlookup",
    "hlookup",
    "linterp",
    "interp",
    "cspline",
    "lspline",
    "pspline",
    "regress",
    "loess",
    "line",
    "slope",
    "intercept",
    "erf",
    "erfc",
    "Gamma",
    "gamma",
    "cumsum",
    "cumprod",
    "tr",
    "eigenvals",
    "eigenvecs",
    "geninv",
    "lu",
    "qr",
    "svd",
    "diag",
    "submatrix",
    "last",
    "csort",
    "rsort",
    "hypot",
    "clamp",
    "polyroots",
    "rkfixed",
    "Rkadapt",
    "fft",
    "ifft",
    "cfft",
    "READPRN",
    "WRITEPRN",
    "READFILE",
    "READEXCEL",
    "num2str",
    "str2num",
    "concat",
    "strlen",
    "substr",
    "search",
    "root",
    "mfill",
    "Fill",
    "rref",
    "lsolve",
    "norm1",
    "normi",
    "norme",
    "cond2",
    "mcond",
    "trace",
    "kronecker",
    "angle",
    "runif",
    "rnorm",
    "dnorm",
    "pnorm",
    "qnorm",
    "binomial",
    "combin",
    "permut",
    "Fresnel",
    "genfit",
    "linfit",
    "expfit",
    "logfit",
    "polyfit",
    "sinfit",
    "medfit",
    "tail",
    "mid",
    "time",
    "tic",
    "toc",
    "isreal",
    "iscomplex",
    "IsArray",
    "IsString",
    "error",
    "until",
    "Dirac",
    "Heaviside",
];

pub fn is_unmapped_function(name: &str) -> bool {
    UNMAPPED_FUNCTIONS.contains(&name)
}

/// Names that begin or are part of a solve block.
pub const SOLVE_NAMES: &[&str] = &[
    "Given", "Find", "Minerr", "Minimize", "Maximize", "Odesolve", "Pfind", "Pminerr",
];

pub fn is_solve_name(name: &str) -> bool {
    SOLVE_NAMES.contains(&name)
}

/// What a built-in constant becomes.
pub enum Constant {
    /// Emitted as given.
    Text(&'static str),
    /// The array origin setting at that point of the sheet.
    Origin,
}

/// Built-in constants that are not units: the name as the source spells it and its Calcpad text.
pub fn lookup_constant(name: &str) -> Option<Constant> {
    match name {
        "π" => Some(Constant::Text("π")),
        "e" => Some(Constant::Text("e")),
        // The source's built-in gravitational acceleration. Calcpad's `g` is the gram.
        "g" => Some(Constant::Text("(9.80665*m/s^2)")),
        "ORIGIN" => Some(Constant::Origin),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn micro_sign_is_normalised() {
        assert_eq!(map_unit("\u{00B5}Pa"), Some("μPa"));
    }

    #[test]
    fn same_and_renamed_units_resolve() {
        assert_eq!(map_unit("kg"), Some("kg"));
        assert_eq!(map_unit("hr"), Some("h"));
        assert_eq!(map_unit("gm"), Some("g"));
        assert_eq!(map_unit("nonsense"), None);
    }

    #[test]
    fn no_duplicate_source_names() {
        let mut seen = std::collections::HashSet::new();
        for u in SAME_UNITS {
            assert!(seen.insert(*u), "duplicate unit {u}");
        }
        for (s, _) in RENAMED_UNITS {
            assert!(seen.insert(*s), "duplicate unit {s}");
        }
        let mut f = std::collections::HashSet::new();
        for (n, ..) in FUNCTIONS {
            assert!(f.insert(*n), "duplicate function {n}");
            assert!(!is_unmapped_function(n), "{n} is both mapped and unmapped");
        }
    }
}
