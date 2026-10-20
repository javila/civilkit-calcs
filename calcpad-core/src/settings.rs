//! Port of Settings.cs (the parts the engine uses).

#[derive(Clone, Debug)]
pub struct MathSettings {
    decimals: i32,
    pub degrees: i32,
    pub is_complex: bool,
    pub substitute: bool,
    pub format_equations: bool,
    pub zero_small_matrix_elements: bool,
    pub show_hidden_output: bool,
    max_output_count: i32,
    pub format_string: Option<String>,
    pub precision: f64,
    pub tol: f64,
}

impl Default for MathSettings {
    fn default() -> Self {
        MathSettings {
            decimals: 2,
            degrees: 0,
            is_complex: false,
            substitute: true,
            format_equations: true,
            zero_small_matrix_elements: true,
            show_hidden_output: false,
            max_output_count: 20,
            format_string: None,
            precision: 1e-14,
            tol: 1e-6,
        }
    }
}

impl MathSettings {
    pub fn decimals(&self) -> i32 {
        self.decimals
    }

    pub fn set_decimals(&mut self, value: i32) {
        self.decimals = value.clamp(0, 15);
    }

    pub fn max_output_count(&self) -> i32 {
        self.max_output_count
    }

    pub fn set_max_output_count(&mut self, value: i32) {
        self.max_output_count = value.clamp(5, 100);
    }

    /// Builder style helpers used by tests.
    pub fn with_decimals(mut self, d: i32) -> Self {
        self.set_decimals(d);
        self
    }
}

#[derive(Clone, Debug)]
pub struct PlotSettings {
    pub is_adaptive: bool,
    pub width: i32,
    pub height: i32,
    pub step: i32,
    pub vector_graphics: bool,
    pub smooth_scale: bool,
    pub shadows: bool,
}

impl Default for PlotSettings {
    fn default() -> Self {
        PlotSettings {
            is_adaptive: true,
            width: 500,
            height: 300,
            step: 0,
            vector_graphics: false,
            smooth_scale: false,
            shadows: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub math: MathSettings,
    pub plot: PlotSettings,
    pub is_us: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            math: MathSettings::default(),
            plot: PlotSettings::default(),
            is_us: true,
        }
    }
}
