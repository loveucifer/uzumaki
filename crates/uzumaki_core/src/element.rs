use cosmic_text::Attrs;
use slotmap::{ new_key_type, SlotMap };
use vello::kurbo::{ Affine, Rect, RoundedRect, Stroke };
use vello::peniko::Color;
use vello::Scene;

use crate::text::TextRenderer;

new_key_type! {
    pub struct NodeId;
}

#[derive(Clone, Debug)]
pub struct ViewProps {
    pub background_color: Color,
    pub border_radius: f64,
    pub border_color: Color,
    pub border_width: f64,
}

impl Default for ViewProps {
    fn default() -> Self {
        Self {
            background_color: Color::TRANSPARENT,
            border_radius: 0.0,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextProps {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
}

#[derive(Clone, Debug)]
pub enum Element {
    Root,
    View(ViewProps),
    Text(TextProps),
}

#[derive(Clone, Debug)]
pub struct NodeContext {
    pub dom_id: NodeId,
    pub text: Option<TextProps>,
}

pub struct Node {
    pub parent: Option<NodeId>,
    pub first_child: Option<NodeId>,
    pub last_child: Option<NodeId>,
    pub next_sibling: Option<NodeId>,
    pub prev_sibling: Option<NodeId>,
    pub taffy_node: taffy::NodeId,
    pub element: Element,
}

pub struct Dom {
    pub nodes: SlotMap<NodeId, Node>,
    pub taffy: taffy::TaffyTree<NodeContext>,
    pub root: Option<NodeId>,
}

impl Dom {
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::with_key(),
            taffy: taffy::TaffyTree::new(),
            root: None,
        }
    }

    pub fn create_element(&mut self, element: Element, style: taffy::Style) -> NodeId {
        let text_context = match &element {
            Element::Text(props) => Some(props.clone()),
            _ => None,
        };
        let taffy_node = self.taffy.new_leaf(style).unwrap();
        let node_id = self.nodes.insert(Node {
            parent: None,
            first_child: None,
            last_child: None,
            next_sibling: None,
            prev_sibling: None,
            taffy_node,
            element,
        });
        self.taffy
            .set_node_context(
                taffy_node,
                Some(NodeContext {
                    dom_id: node_id,
                    text: text_context,
                })
            )
            .unwrap();
        node_id
    }

    pub fn set_root(&mut self, node_id: NodeId) {
        self.root = Some(node_id);
    }

    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        // Sync taffy tree
        let parent_taffy = self.nodes[parent_id].taffy_node;
        let child_taffy = self.nodes[child_id].taffy_node;
        self.taffy.add_child(parent_taffy, child_taffy).unwrap();

        // Update linked list
        let old_last = self.nodes[parent_id].last_child;
        self.nodes[child_id].parent = Some(parent_id);
        self.nodes[child_id].prev_sibling = old_last;
        self.nodes[child_id].next_sibling = None;

        if let Some(old_last_id) = old_last {
            self.nodes[old_last_id].next_sibling = Some(child_id);
        } else {
            self.nodes[parent_id].first_child = Some(child_id);
        }
        self.nodes[parent_id].last_child = Some(child_id);
    }

    pub fn insert_before(&mut self, parent_id: NodeId, child_id: NodeId, before_id: NodeId) {
        // Sync taffy tree
        let parent_taffy = self.nodes[parent_id].taffy_node;
        let child_taffy = self.nodes[child_id].taffy_node;
        let before_taffy = self.nodes[before_id].taffy_node;

        let children = self.taffy.children(parent_taffy).unwrap();
        let idx = children
            .iter()
            .position(|&c| c == before_taffy)
            .expect("before node not found in parent");
        self.taffy.insert_child_at_index(parent_taffy, idx, child_taffy).unwrap();

        // Update linked list
        let prev = self.nodes[before_id].prev_sibling;
        self.nodes[child_id].parent = Some(parent_id);
        self.nodes[child_id].next_sibling = Some(before_id);
        self.nodes[child_id].prev_sibling = prev;
        self.nodes[before_id].prev_sibling = Some(child_id);

        if let Some(prev_id) = prev {
            self.nodes[prev_id].next_sibling = Some(child_id);
        } else {
            self.nodes[parent_id].first_child = Some(child_id);
        }
    }

    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        // Sync taffy tree
        let parent_taffy = self.nodes[parent_id].taffy_node;
        let child_taffy = self.nodes[child_id].taffy_node;
        self.taffy.remove_child(parent_taffy, child_taffy).unwrap();

        // Update linked list
        let prev = self.nodes[child_id].prev_sibling;
        let next = self.nodes[child_id].next_sibling;

        if let Some(prev_id) = prev {
            self.nodes[prev_id].next_sibling = next;
        } else {
            self.nodes[parent_id].first_child = next;
        }

        if let Some(next_id) = next {
            self.nodes[next_id].prev_sibling = prev;
        } else {
            self.nodes[parent_id].last_child = prev;
        }

        self.nodes[child_id].parent = None;
        self.nodes[child_id].prev_sibling = None;
        self.nodes[child_id].next_sibling = None;
    }

    pub fn compute_layout(&mut self, width: f32, height: f32, text_renderer: &mut TextRenderer) {
        if let Some(root) = self.root {
            let taffy_root = self.nodes[root].taffy_node;
            self.taffy
                .compute_layout_with_measure(
                    taffy_root,
                    taffy::Size {
                        width: taffy::AvailableSpace::Definite(width),
                        height: taffy::AvailableSpace::Definite(height),
                    },
                    |known_dimensions, available_space, _node_id, node_context, _style| {
                        Self::measure(
                            text_renderer,
                            known_dimensions,
                            available_space,
                            node_context
                        )
                    }
                )
                .unwrap();
        }
    }

    pub fn render(&self, scene: &mut Scene, text_renderer: &mut TextRenderer) {
        if let Some(root) = self.root {
            self.render_node(scene, text_renderer, root, 0.0, 0.0);
        }
    }

    fn render_node(
        &self,
        scene: &mut Scene,
        text_renderer: &mut TextRenderer,
        node_id: NodeId,
        parent_x: f64,
        parent_y: f64
    ) {
        let node = &self.nodes[node_id];
        let Ok(layout) = self.taffy.layout(node.taffy_node) else {
            return;
        };

        let x = parent_x + (layout.location.x as f64);
        let y = parent_y + (layout.location.y as f64);
        let w = layout.size.width as f64;
        let h = layout.size.height as f64;

        match &node.element {
            Element::View(props) => {
                if props.border_radius > 0.0 {
                    let shape = RoundedRect::from_rect(
                        Rect::new(x, y, x + w, y + h),
                        props.border_radius
                    );
                    scene.fill(
                        vello::peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        props.background_color,
                        None,
                        &shape
                    );
                    if props.border_width > 0.0 {
                        scene.stroke(
                            &Stroke::new(props.border_width),
                            Affine::IDENTITY,
                            props.border_color,
                            None,
                            &shape
                        );
                    }
                } else {
                    let shape = Rect::new(x, y, x + w, y + h);
                    scene.fill(
                        vello::peniko::Fill::NonZero,
                        Affine::IDENTITY,
                        props.background_color,
                        None,
                        &shape
                    );
                    if props.border_width > 0.0 {
                        scene.stroke(
                            &Stroke::new(props.border_width),
                            Affine::IDENTITY,
                            props.border_color,
                            None,
                            &shape
                        );
                    }
                }
            }
            Element::Root => {}
            Element::Text(props) => {
                text_renderer.draw_text(
                    scene,
                    &props.content,
                    Attrs::new(),
                    props.font_size,
                    w as f32,
                    h as f32,
                    (x as f32, y as f32),
                    props.color
                );
            }
        }

        // Traverse children via linked list
        let mut child = node.first_child;
        while let Some(child_id) = child {
            self.render_node(scene, text_renderer, child_id, x, y);
            child = self.nodes[child_id].next_sibling;
        }
    }

    fn measure(
        text_renderer: &mut TextRenderer,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        node_context: Option<&mut NodeContext>
    ) -> taffy::Size<f32> {
        let default_size = taffy::Size {
            width: known_dimensions.width.unwrap_or(0.0),
            height: known_dimensions.height.unwrap_or(0.0),
        };

        let Some(ctx) = node_context else {
            return default_size;
        };

        if let Some(text) = &ctx.text {
            let (measured_width, measured_height) = text_renderer.measure_text(
                &text.content,
                Attrs::new(),
                text.font_size,
                known_dimensions.width.or_else(|| available_as_option(available_space.width)),
                known_dimensions.height.or_else(|| available_as_option(available_space.height))
            );

            return taffy::Size {
                width: measured_width,
                height: measured_height,
            };
        }

        default_size
    }
}

// Helpers for uniform taffy geometry
fn available_as_option(space: taffy::AvailableSpace) -> Option<f32> {
    match space {
        taffy::AvailableSpace::Definite(v) => Some(v),
        _ => None,
    }
}

fn length_rect(val: f32) -> taffy::Rect<taffy::LengthPercentage> {
    let v = taffy::LengthPercentage::length(val);
    taffy::Rect {
        left: v,
        right: v,
        top: v,
        bottom: v,
    }
}

fn length_size(val: f32) -> taffy::Size<taffy::LengthPercentage> {
    let v = taffy::LengthPercentage::length(val);
    taffy::Size {
        width: v,
        height: v,
    }
}

/// Builds a hardcoded demo UI tree: dark-themed dashboard with text.
pub fn build_demo_tree() -> Dom {
    use taffy::*;

    let mut dom = Dom::new();

    // VS Code Dark+ inspired palette
    let base = Color::from_rgba8(15, 15, 15, 255); // main background
    let panel = Color::from_rgba8(20, 20, 20, 255); // surfaces/cards
    let border = Color::from_rgba8(60, 60, 60, 255); // subtle dividers
    let text_color = Color::from_rgba8(212, 212, 212, 255); // primary text
    let subtext = Color::from_rgba8(140, 140, 150, 255); // secondary text
    let accent_blue = Color::from_rgba8(86, 156, 214, 255); // keyword blue
    let accent_green = Color::from_rgba8(102, 204, 153, 255);
    let accent_orange = Color::from_rgba8(206, 145, 120, 255);
    let nav_active = Color::from_rgba8(45, 45, 48, 255); // selected item

    // Root
    let root = dom.create_element(Element::Root, Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        size: Size {
            width: Dimension::percent(1.0),
            height: Dimension::percent(1.0),
        },
        ..Default::default()
    });
    dom.set_root(root);

    // Header
    let header = dom.create_element(
        Element::View(ViewProps {
            background_color: panel,
            border_color: border,
            border_width: 1.0,
            ..Default::default()
        }),
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::auto(),
                height: Dimension::length(48.0),
            },
            padding: length_rect(16.0),
            ..Default::default()
        }
    );
    dom.append_child(root, header);

    let header_text = dom.create_element(
        Element::Text(TextProps {
            content: "Uzumaki".to_string(),
            font_size: 18.0,
            color: accent_blue,
        }),
        Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            flex_shrink: 0.0,
            ..Default::default()
        }
    );
    dom.append_child(header, header_text);

    // Body
    let body = dom.create_element(
        Element::View(ViewProps {
            background_color: base,
            ..Default::default()
        }),
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            flex_grow: 1.0,
            ..Default::default()
        }
    );
    dom.append_child(root, body);

    // Sidebar
    let sidebar = dom.create_element(
        Element::View(ViewProps {
            background_color: panel,
            border_color: border,
            border_width: 1.0,
            ..Default::default()
        }),
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            size: Size {
                width: Dimension::length(400.0),
                height: Dimension::auto(),
            },
            padding: length_rect(12.0),
            gap: length_size(4.0),
            ..Default::default()
        }
    );
    dom.append_child(body, sidebar);

    // Sidebar nav items
    let nav_labels = ["Dashboard", "Analytics", "Projects", "Settings"];
    for (i, label) in nav_labels.iter().enumerate() {
        let nav = dom.create_element(
            Element::View(ViewProps {
                background_color: if i == 0 {
                    nav_active
                } else {
                    Color::TRANSPARENT
                },
                border_radius: 6.0,
                ..Default::default()
            }),
            Style {
                display: Display::Flex,
                align_items: Some(AlignItems::Center),
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::length(36.0),
                },
                padding: length_rect(8.0),
                flex_shrink: 0.0,
                ..Default::default()
            }
        );
        dom.append_child(sidebar, nav);

        let nav_text = dom.create_element(
            Element::Text(TextProps {
                content: label.to_string(),
                font_size: 20.0,
                color: if i == 0 {
                    text_color
                } else {
                    subtext
                },
            }),
            Style {
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::auto(),
                },
                ..Default::default()
            }
        );
        dom.append_child(nav, nav_text);
    }

    // Main content area
    let main_area = dom.create_element(Element::Root, Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Column,
        flex_grow: 1.0,
        padding: length_rect(16.0),
        gap: length_size(16.0),
        ..Default::default()
    });
    dom.append_child(body, main_area);

    // Page title
    let page_title = dom.create_element(
        Element::Text(TextProps {
            content: "Dashboard".to_string(),
            font_size: 22.0,
            color: text_color,
        }),
        Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            ..Default::default()
        }
    );
    dom.append_child(main_area, page_title);

    // Top card row
    let card_row = dom.create_element(Element::Root, Style {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        gap: length_size(12.0),
        size: Size {
            width: Dimension::auto(),
            height: Dimension::length(100.0),
        },
        ..Default::default()
    });
    dom.append_child(main_area, card_row);

    // Three metric cards
    let cards = [
        ("Revenue", "$12,400", accent_blue),
        ("Users", "1,240", accent_green),
        ("Growth", "+24%", accent_orange),
    ];
    for (title, value, accent) in cards {
        let card = dom.create_element(
            Element::View(ViewProps {
                background_color: panel,
                border_radius: 8.0,
                border_color: border,
                border_width: 1.0,
            }),
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                padding: length_rect(16.0),
                gap: length_size(8.0),
                ..Default::default()
            }
        );
        dom.append_child(card_row, card);

        let card_title = dom.create_element(
            Element::Text(TextProps {
                content: title.to_string(),
                font_size: 16.0,
                color: subtext,
            }),
            Style {
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::auto(),
                },
                ..Default::default()
            }
        );
        dom.append_child(card, card_title);

        let card_value = dom.create_element(
            Element::Text(TextProps {
                content: value.to_string(),
                font_size: 24.0,
                color: accent,
            }),
            Style {
                size: Size {
                    width: Dimension::auto(),
                    height: Dimension::auto(),
                },
                ..Default::default()
            }
        );
        dom.append_child(card, card_value);
    }

    // Bottom panel
    let bottom = dom.create_element(
        Element::View(ViewProps {
            background_color: panel,
            border_radius: 8.0,
            border_color: border,
            border_width: 1.0,
        }),
        Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            padding: length_rect(16.0),
            gap: length_size(8.0),
            ..Default::default()
        }
    );
    dom.append_child(main_area, bottom);

    let panel_title = dom.create_element(
        Element::Text(TextProps {
            content: "Recent Activity".to_string(),
            font_size: 16.0,
            color: text_color,
        }),
        Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            ..Default::default()
        }
    );
    dom.append_child(bottom, panel_title);

    let panel_text = dom.create_element(
        Element::Text(TextProps {
            content: "No recent activity to display.".to_string(),
            font_size: 16.0,
            color: subtext,
        }),
        Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            ..Default::default()
        }
    );
    dom.append_child(bottom, panel_text);

    // Footer
    let footer = dom.create_element(
        Element::View(ViewProps {
            background_color: panel,
            border_color: border,
            border_width: 1.0,
            ..Default::default()
        }),
        Style {
            display: Display::Flex,
            align_items: Some(AlignItems::Center),
            size: Size {
                width: Dimension::auto(),
                height: Dimension::length(32.0),
            },
            padding: length_rect(16.0),
            ..Default::default()
        }
    );
    dom.append_child(root, footer);

    let footer_text = dom.create_element(
        Element::Text(TextProps {
            content: "Uzumaki v0.1.0".to_string(),
            font_size: 16.0,
            color: subtext,
        }),
        Style {
            size: Size {
                width: Dimension::auto(),
                height: Dimension::auto(),
            },
            ..Default::default()
        }
    );
    dom.append_child(footer, footer_text);

    dom
}
