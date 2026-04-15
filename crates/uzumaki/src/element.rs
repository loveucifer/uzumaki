use crate::cursor::UzCursorIcon;
use crate::input::BaseInputState;
use crate::interactivity::Interactivity;
use crate::style::{Bounds, TextSelectable, UzStyle};

pub mod input;
pub mod render;
pub mod selection;
pub mod text;
pub mod view;

pub use selection::{DomRangeProvider, SharedSelectionState};
use vello::kurbo::Affine;

pub type InputState = BaseInputState<DomRangeProvider>;

pub type UzNodeId = usize;

pub struct ScrollState {
    pub scroll_offset_y: f32,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            scroll_offset_y: 0.0,
        }
    }
}

/// Active scroll-thumb drag. Stored on Dom (only one drag at a time).
pub struct ScrollDragState {
    pub node_id: UzNodeId,
    pub start_mouse_y: f64,
    pub start_scroll_offset: f32,
    /// Track length = visible_height - thumb_height (how far thumb can move).
    pub track_range: f64,
    /// Max scroll offset (content_height - visible_height).
    pub max_scroll: f32,
}

/// Rendered thumb rect, rebuilt each paint pass for hit testing.
pub struct ScrollThumbRect {
    pub node_id: UzNodeId,
    pub thumb_bounds: Bounds,
    pub view_bounds: Bounds,
    pub content_height: f32,
    pub visible_height: f32,
}

#[derive(Clone, Debug)]
pub struct TextContent {
    pub content: String,
}

// ── Inherited properties ─────────────────────────────────────────────
// General-purpose mechanism for properties that propagate from parent to child
// unless explicitly overridden. Designed for extension — future inheritable
// properties (font color, font size, line height, etc.) go here.

#[derive(Clone, Debug, Default)]
pub struct InheritedProperties {
    pub text_selectable: bool,
}

// ── View text selection ──────────────────────────────────────────────

/// One text node's contribution to a textSelect run.
pub struct TextRunEntry {
    pub node_id: UzNodeId,
    /// Start grapheme index of this node in the flat run.
    pub flat_start: usize,
    pub grapheme_count: usize,
}

/// The complete text run for a textSelect subtree.
/// Built each frame; maps between flat grapheme offsets and per-node positions.
pub struct TextSelectRun {
    pub root_id: UzNodeId,
    pub entries: Vec<TextRunEntry>,
    pub flat_text: String,
    pub total_graphemes: usize,
}

// ── Element trait ──────────────────────────────────────────────────────

pub trait ElementBehavior {
    fn as_input(&self) -> Option<&InputState> {
        None
    }
    fn as_input_mut(&mut self) -> Option<&mut InputState> {
        None
    }
    fn as_text(&self) -> Option<&TextContent> {
        None
    }
    fn as_text_mut(&mut self) -> Option<&mut TextContent> {
        None
    }
    fn is_input(&self) -> bool {
        false
    }
    fn is_text(&self) -> bool {
        false
    }

    /// Default cursor for this behavior when unset by style.
    fn default_cursor(&self) -> Option<UzCursorIcon> {
        None
    }
}

pub struct ViewBehavior;
impl ElementBehavior for ViewBehavior {}

pub struct TextBehavior {
    pub content: TextContent,
}

impl ElementBehavior for TextBehavior {
    fn as_text(&self) -> Option<&TextContent> {
        Some(&self.content)
    }
    fn as_text_mut(&mut self) -> Option<&mut TextContent> {
        Some(&mut self.content)
    }
    fn is_text(&self) -> bool {
        true
    }
}

pub struct InputBehavior {
    pub state: InputState,
}

impl InputBehavior {
    pub fn new(state: InputState) -> Self {
        Self { state }
    }

    pub fn new_single_line(mut state: InputState) -> Self {
        state.multiline = false;
        Self::new(state)
    }
}

impl ElementBehavior for InputBehavior {
    fn as_input(&self) -> Option<&InputState> {
        Some(&self.state)
    }
    fn as_input_mut(&mut self) -> Option<&mut InputState> {
        Some(&mut self.state)
    }
    fn is_input(&self) -> bool {
        true
    }
    fn default_cursor(&self) -> Option<UzCursorIcon> {
        if self.state.disabled {
            Some(UzCursorIcon::NotAllowed)
        } else {
            Some(UzCursorIcon::Text)
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeContext {
    pub dom_id: UzNodeId,
    pub text: Option<TextContent>,
    pub font_size: f32,
    pub is_input: bool,
}

pub struct Node {
    pub parent: Option<UzNodeId>,
    pub first_child: Option<UzNodeId>,
    pub last_child: Option<UzNodeId>,
    pub next_sibling: Option<UzNodeId>,
    pub prev_sibling: Option<UzNodeId>,
    pub taffy_node: taffy::NodeId,
    pub behavior: Box<dyn ElementBehavior>,
    /// The base style for this element. Converted to taffy for layout.
    pub style: UzStyle,
    /// Interactivity: hover/active style overrides, hitbox, event listeners.
    pub interactivity: Interactivity,
    /// Scroll state, present only when overflow_y == Scroll.
    pub scroll_state: Option<ScrollState>,
    // not used now todo use this :3
    pub transform: Option<Affine>,
}

impl Node {
    pub fn new(
        taffy_node: taffy::NodeId,
        style: UzStyle,
        behavior: impl ElementBehavior + 'static,
    ) -> Self {
        Self {
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            taffy_node,
            behavior: Box::new(behavior),
            style,
            interactivity: Interactivity::new(),
            scroll_state: None,
            transform: None,
        }
    }
}

impl Node {
    #[inline]
    pub fn text_selectable(&self) -> TextSelectable {
        self.style.text_selectable
    }

    pub fn is_text_selectable(&self) -> bool {
        self.style.text_selectable.selectable()
    }

    pub fn set_text_selectable(&mut self, text_selectable: TextSelectable) {
        self.style.text_selectable = text_selectable
    }
}
