//! The fonts resvg draws an SVG's text with. The browser has no system fonts to offer the
//! engine, so the page passes the font files in (`add_font`); the CLI can also load the
//! system's (`load_system_fonts`, feature `system-fonts`). Generic "sans" is DejaVu Sans when
//! it is loaded.
#[cfg(feature = "svg")]
use resvg::usvg::fontdb::Database;
#[cfg(feature = "svg")]
use std::cell::RefCell;

#[cfg(feature = "svg")]
thread_local! {
    static DB: RefCell<Database> = RefCell::new(Database::new());
}

/// Adds a font file (TTF, OTF or TTC).
#[cfg(feature = "svg")]
pub fn add_font(data: Vec<u8>) {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.load_font_data(data);
        db.set_sans_serif_family("DejaVu Sans");
    });
}

#[cfg(feature = "system-fonts")]
pub fn load_system_fonts() {
    DB.with(|db| {
        let mut db = db.borrow_mut();
        db.load_system_fonts();
        let has = |f: &str| db.faces().any(|x| x.families.iter().any(|(n, _)| n == f));
        if let Some(f) = ["DejaVu Sans", "Segoe UI", "Arial", "Helvetica", "Liberation Sans"]
            .into_iter()
            .find(|f| has(f))
        {
            db.set_sans_serif_family(f);
        }
    });
}

#[cfg(feature = "svg")]
pub fn font_count() -> usize {
    DB.with(|db| db.borrow().len())
}

#[cfg(feature = "svg")]
pub(crate) fn with_database<F: FnOnce(&Database)>(f: F) {
    DB.with(|db| f(&db.borrow()));
}

#[cfg(not(feature = "svg"))]
pub fn add_font(_data: Vec<u8>) {}
