use crate::app::selection::TextSelection;
use crate::app::PreviewContent;
use ratatui::prelude::Rect;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
pub struct Pane {
    pub id: usize,
    pub file_path: Option<PathBuf>,
    pub preview_content: Option<PreviewContent>,
    pub scroll: u16,
    pub selection: Option<TextSelection>,
}

impl Pane {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            file_path: None,
            preview_content: None,
            scroll: 0,
            selection: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum SplitNode {
    Leaf {
        pane_index: usize,
    },
    Horizontal {
        left: Box<SplitNode>,
        right: Box<SplitNode>,
        percent: u16,
    },
    Vertical {
        top: Box<SplitNode>,
        bottom: Box<SplitNode>,
        percent: u16,
    },
}

pub struct SplitLayout {
    pub panes: Vec<Pane>,
    pub split_tree: SplitNode,
    pub active_pane_index: usize,
    pub file_list_visible: bool,
    next_pane_id: usize,
}

impl SplitLayout {
    pub fn new() -> Self {
        Self {
            panes: vec![Pane::new(0)],
            split_tree: SplitNode::Leaf { pane_index: 0 },
            active_pane_index: 0,
            file_list_visible: true,
            next_pane_id: 1,
        }
    }

    pub fn split_active_pane(
        &mut self,
        direction: SplitDirection,
        default_percent: u16,
    ) -> anyhow::Result<()> {
        if self.panes.len() >= 4 {
            anyhow::bail!("Maximum of 4 panes reached");
        }

        let new_pane_id = self.next_pane_id;
        self.next_pane_id += 1;

        // Clone active pane content for the new pane
        let mut new_pane = Pane::new(new_pane_id);
        if let Some(active_pane) = self.panes.iter().find(|p| p.id == self.active_pane_index) {
            new_pane.file_path = active_pane.file_path.clone();
            new_pane.preview_content = active_pane.preview_content.clone();
            new_pane.scroll = active_pane.scroll;
        }

        self.panes.push(new_pane);
        let new_leaf = SplitNode::Leaf {
            pane_index: new_pane_id,
        };

        self.split_tree = self.update_tree_with_split(
            self.split_tree.clone(),
            self.active_pane_index,
            new_leaf,
            direction,
            default_percent,
        );

        self.active_pane_index = new_pane_id;
        Ok(())
    }

    fn update_tree_with_split(
        &self,
        node: SplitNode,
        target_id: usize,
        new_leaf: SplitNode,
        direction: SplitDirection,
        percent: u16,
    ) -> SplitNode {
        match node {
            SplitNode::Leaf { pane_index } if pane_index == target_id => match direction {
                SplitDirection::Left => SplitNode::Horizontal {
                    left: Box::new(new_leaf),
                    right: Box::new(node),
                    percent,
                },
                SplitDirection::Right => SplitNode::Horizontal {
                    left: Box::new(node),
                    right: Box::new(new_leaf),
                    percent,
                },
                SplitDirection::Up => SplitNode::Vertical {
                    top: Box::new(new_leaf),
                    bottom: Box::new(node),
                    percent,
                },
                SplitDirection::Down => SplitNode::Vertical {
                    top: Box::new(node),
                    bottom: Box::new(new_leaf),
                    percent,
                },
            },
            SplitNode::Horizontal {
                left,
                right,
                percent: p,
            } => SplitNode::Horizontal {
                left: Box::new(self.update_tree_with_split(
                    *left,
                    target_id,
                    new_leaf.clone(),
                    direction,
                    percent,
                )),
                right: Box::new(
                    self.update_tree_with_split(*right, target_id, new_leaf, direction, percent),
                ),
                percent: p,
            },
            SplitNode::Vertical {
                top,
                bottom,
                percent: p,
            } => SplitNode::Vertical {
                top: Box::new(self.update_tree_with_split(
                    *top,
                    target_id,
                    new_leaf.clone(),
                    direction,
                    percent,
                )),
                bottom: Box::new(
                    self.update_tree_with_split(*bottom, target_id, new_leaf, direction, percent),
                ),
                percent: p,
            },
            _ => node,
        }
    }

    pub fn close_active_pane(&mut self) {
        if self.panes.len() <= 1 {
            return;
        }

        let id_to_remove = self.active_pane_index;
        self.split_tree = self
            .remove_from_tree(self.split_tree.clone(), id_to_remove)
            .unwrap_or(SplitNode::Leaf { pane_index: 0 });
        self.panes.retain(|p| p.id != id_to_remove);

        // Set active to the first remaining pane
        if let Some(first) = self.panes.first() {
            self.active_pane_index = first.id;
        }
    }

    fn remove_from_tree(&self, node: SplitNode, target_id: usize) -> Option<SplitNode> {
        match node {
            SplitNode::Leaf { pane_index } => {
                if pane_index == target_id {
                    None
                } else {
                    Some(node)
                }
            }
            SplitNode::Horizontal {
                left,
                right,
                percent,
            } => {
                let new_left = self.remove_from_tree(*left, target_id);
                let new_right = self.remove_from_tree(*right, target_id);
                match (new_left, new_right) {
                    (None, Some(r)) => Some(r),
                    (Some(l), None) => Some(l),
                    (Some(l), Some(r)) => Some(SplitNode::Horizontal {
                        left: Box::new(l),
                        right: Box::new(r),
                        percent,
                    }),
                    (None, None) => None,
                }
            }
            SplitNode::Vertical {
                top,
                bottom,
                percent,
            } => {
                let new_top = self.remove_from_tree(*top, target_id);
                let new_bottom = self.remove_from_tree(*bottom, target_id);
                match (new_top, new_bottom) {
                    (None, Some(b)) => Some(b),
                    (Some(t), None) => Some(t),
                    (Some(t), Some(b)) => Some(SplitNode::Vertical {
                        top: Box::new(t),
                        bottom: Box::new(b),
                        percent,
                    }),
                    (None, None) => None,
                }
            }
        }
    }

    pub fn cycle_active_pane(&mut self) {
        if self.panes.is_empty() {
            return;
        }
        let current_idx = self
            .panes
            .iter()
            .position(|p| p.id == self.active_pane_index)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.panes.len();
        self.active_pane_index = self.panes[next_idx].id;
    }

    pub fn swap_active_split_orientation(&mut self) {
        self.split_tree = self.swap_orientation(self.split_tree.clone(), self.active_pane_index);
    }

    fn swap_orientation(&self, node: SplitNode, target_id: usize) -> SplitNode {
        match node {
            SplitNode::Leaf { .. } => node,
            SplitNode::Horizontal {
                left,
                right,
                percent,
            } => {
                if self.contains_pane(&left, target_id) || self.contains_pane(&right, target_id) {
                    SplitNode::Vertical {
                        top: left,
                        bottom: right,
                        percent,
                    }
                } else {
                    SplitNode::Horizontal {
                        left: Box::new(self.swap_orientation(*left, target_id)),
                        right: Box::new(self.swap_orientation(*right, target_id)),
                        percent,
                    }
                }
            }
            SplitNode::Vertical {
                top,
                bottom,
                percent,
            } => {
                if self.contains_pane(&top, target_id) || self.contains_pane(&bottom, target_id) {
                    SplitNode::Horizontal {
                        left: top,
                        right: bottom,
                        percent,
                    }
                } else {
                    SplitNode::Vertical {
                        top: Box::new(self.swap_orientation(*top, target_id)),
                        bottom: Box::new(self.swap_orientation(*bottom, target_id)),
                        percent,
                    }
                }
            }
        }
    }

    fn contains_pane(&self, node: &SplitNode, target_id: usize) -> bool {
        match node {
            SplitNode::Leaf { pane_index } => *pane_index == target_id,
            SplitNode::Horizontal { left, right, .. } => {
                self.contains_pane(left, target_id) || self.contains_pane(right, target_id)
            }
            SplitNode::Vertical { top, bottom, .. } => {
                self.contains_pane(top, target_id) || self.contains_pane(bottom, target_id)
            }
        }
    }

    pub fn resize_active_split(&mut self, delta: i16) {
        self.split_tree = self.resize_tree(self.split_tree.clone(), self.active_pane_index, delta);
    }

    fn resize_tree(&self, node: SplitNode, target_id: usize, delta: i16) -> SplitNode {
        match node {
            SplitNode::Leaf { .. } => node,
            SplitNode::Horizontal {
                left,
                right,
                percent,
            } => {
                if self.contains_pane(&left, target_id) || self.contains_pane(&right, target_id) {
                    let new_percent = (percent as i16 + delta).clamp(10, 90) as u16;
                    SplitNode::Horizontal {
                        left,
                        right,
                        percent: new_percent,
                    }
                } else {
                    SplitNode::Horizontal {
                        left: Box::new(self.resize_tree(*left, target_id, delta)),
                        right: Box::new(self.resize_tree(*right, target_id, delta)),
                        percent,
                    }
                }
            }
            SplitNode::Vertical {
                top,
                bottom,
                percent,
            } => {
                if self.contains_pane(&top, target_id) || self.contains_pane(&bottom, target_id) {
                    let new_percent = (percent as i16 + delta).clamp(10, 90) as u16;
                    SplitNode::Vertical {
                        top,
                        bottom,
                        percent: new_percent,
                    }
                } else {
                    SplitNode::Vertical {
                        top: Box::new(self.resize_tree(*top, target_id, delta)),
                        bottom: Box::new(self.resize_tree(*bottom, target_id, delta)),
                        percent,
                    }
                }
            }
        }
    }

    pub fn get_pane_areas(&self, area: Rect) -> Vec<(usize, Rect)> {
        let mut areas = Vec::new();
        self.collect_areas(&self.split_tree, area, &mut areas);
        areas
    }

    fn collect_areas(&self, node: &SplitNode, area: Rect, areas: &mut Vec<(usize, Rect)>) {
        match node {
            SplitNode::Leaf { pane_index } => {
                areas.push((*pane_index, area));
            }
            SplitNode::Horizontal {
                left,
                right,
                percent,
            } => {
                let left_width = (area.width as u32 * *percent as u32 / 100) as u16;
                let left_area = Rect::new(area.x, area.y, left_width, area.height);
                let right_area = Rect::new(
                    area.x + left_width,
                    area.y,
                    area.width - left_width,
                    area.height,
                );
                self.collect_areas(left, left_area, areas);
                self.collect_areas(right, right_area, areas);
            }
            SplitNode::Vertical {
                top,
                bottom,
                percent,
            } => {
                let top_height = (area.height as u32 * *percent as u32 / 100) as u16;
                let top_area = Rect::new(area.x, area.y, area.width, top_height);
                let bottom_area = Rect::new(
                    area.x,
                    area.y + top_height,
                    area.width,
                    area.height - top_height,
                );
                self.collect_areas(top, top_area, areas);
                self.collect_areas(bottom, bottom_area, areas);
            }
        }
    }
}
