use winit::window::CursorIcon as WinitCursorIcon;

/// A subset of CSS cursor keywords that map cleanly to platform cursors.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum UzCursorIcon {
    #[default]
    Default,
    Pointer,
    Text,
    Wait,
    Crosshair,
    Move,
    NotAllowed,
    Grab,
    Grabbing,
    Help,
    Progress,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize,
    AllScroll,
    ZoomIn,
    ZoomOut,
}

impl UzCursorIcon {
    pub fn parse(value: &str) -> Option<Self> {
        let icon = match value.trim() {
            "default" | "auto" | "initial" => Self::Default,
            "pointer" | "hand" => Self::Pointer,
            "text" | "caret" | "i-beam" => Self::Text,
            "wait" => Self::Wait,
            "crosshair" => Self::Crosshair,
            "move" => Self::Move,
            "not-allowed" | "no-drop" => Self::NotAllowed,
            "grab" => Self::Grab,
            "grabbing" => Self::Grabbing,
            "help" => Self::Help,
            "progress" => Self::Progress,
            "ew-resize" | "col-resize" => Self::EwResize,
            "ns-resize" | "row-resize" => Self::NsResize,
            "nesw-resize" => Self::NeswResize,
            "nwse-resize" => Self::NwseResize,
            "all-scroll" => Self::AllScroll,
            "zoom-in" => Self::ZoomIn,
            "zoom-out" => Self::ZoomOut,
            _ => return None,
        };
        Some(icon)
    }

    pub fn to_winit(self) -> WinitCursorIcon {
        match self {
            Self::Default => WinitCursorIcon::Default,
            Self::Pointer => WinitCursorIcon::Pointer,
            Self::Text => WinitCursorIcon::Text,
            Self::Wait => WinitCursorIcon::Wait,
            Self::Crosshair => WinitCursorIcon::Crosshair,
            Self::Move => WinitCursorIcon::Move,
            Self::NotAllowed => WinitCursorIcon::NotAllowed,
            Self::Grab => WinitCursorIcon::Grab,
            Self::Grabbing => WinitCursorIcon::Grabbing,
            Self::Help => WinitCursorIcon::Help,
            Self::Progress => WinitCursorIcon::Progress,
            Self::EwResize => WinitCursorIcon::EwResize,
            Self::NsResize => WinitCursorIcon::NsResize,
            Self::NeswResize => WinitCursorIcon::NeswResize,
            Self::NwseResize => WinitCursorIcon::NwseResize,
            Self::ColResize => WinitCursorIcon::ColResize,
            Self::RowResize => WinitCursorIcon::RowResize,
            Self::AllScroll => WinitCursorIcon::AllScroll,
            Self::ZoomIn => WinitCursorIcon::ZoomIn,
            Self::ZoomOut => WinitCursorIcon::ZoomOut,
        }
    }
}
