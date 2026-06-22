//! Permission mode shared by connectors. A connector reads its `<NAME>_MODE`
//! env var; in Viewer mode it removes its write tools from the router (via
//! `ToolRouter::remove_route`), so Claude only ever sees read-only tools.

use crate::env::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Read-only: write tools are removed from the router.
    Viewer,
    /// Full access: all tools available.
    Writer,
}

impl Mode {
    /// Read the mode from an env var. Anything matching viewer/read/readonly is
    /// Viewer; everything else (including unset) defaults to Writer.
    pub fn from_env(var: &str) -> Self {
        match env(var).as_deref().map(str::trim) {
            Some(v)
                if v.eq_ignore_ascii_case("viewer")
                    || v.eq_ignore_ascii_case("read")
                    || v.eq_ignore_ascii_case("readonly")
                    || v.eq_ignore_ascii_case("read-only") =>
            {
                Mode::Viewer
            }
            _ => Mode::Writer,
        }
    }

    pub fn is_viewer(self) -> bool {
        matches!(self, Mode::Viewer)
    }
}
