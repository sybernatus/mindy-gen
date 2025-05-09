use crate::layout::Layout;
use crate::layout::pos2::Pos2;
use crate::layout::size::Size;
use crate::mindmap::Mindmap;
use crate::mindmap::style::MindmapStyle;
use crate::node::Node;

pub struct LeftRightHorizontalLayout {}

impl Layout for LeftRightHorizontalLayout {}

impl LeftRightHorizontalLayout {


    /// Recursively place the node positions based on the parent position and size
    /// # Arguments
    /// * `node` - A mutable reference to the node
    /// * `parent_position` - The position of the parent node
    /// * `parent_size` - The size of the parent node
    /// * `side` - The side of the tree (-1 for left, 1 for right)
    /// * `horizontal_padding` - The horizontal padding between nodes
    /// * `vertical_padding` - The vertical padding between nodes
    /// # Returns
    /// * `f32` - The total height of the subtree
    fn place_node_positions(
        node: &mut Node,
        parent_position: Pos2,
        parent_size: Size,
        side: f32,
        horizontal_padding: f32,
        vertical_padding: f32,
    ) -> f32 {
        let mut total_height = 0.0;
        let node_size = node.graphical_size.clone().unwrap_or(Size { width: 0.0, height: 0.0 });
        let node_position_x = parent_position.x + side * (node_size.width / 2.0 + horizontal_padding + parent_size.width / 2.0);
        node.position_from_initial = Some(Pos2::new(node_position_x, parent_position.clone().y));

        if let Some(children) = &mut node.children {
            let subtree_height = node.children_graphical_size.clone().unwrap().height;
            let mut y_cursor = parent_position.y - subtree_height / 2.0;

            for child in children.iter_mut() {
                 let child_size = child.graphical_size.clone().unwrap_or(Size { width: 0.0, height: 0.0 });
                let child_subtree = child.children_graphical_size.clone().unwrap_or(child_size.clone());

                let child_y = y_cursor + child_subtree.height / 2.0;

                let child_offset = Pos2 { x: node_position_x, y: child_y };
                child.position_from_initial = Some(child_offset.clone());
                Self::place_node_positions(
                    child,
                    child_offset,
                    node_size.clone(),
                    side,
                    horizontal_padding,
                    vertical_padding,
                );

                y_cursor += child_subtree.height + vertical_padding;
                total_height = subtree_height.max(total_height);
            }
        }
        total_height
    }

    /// Recursively place the node positions based on the parent position and size
    /// # Arguments
    /// * `element_tree` - A mutable reference to a vector of nodes
    /// * `padding_horizontal` - The horizontal padding between nodes
    /// * `padding_vertical` - The vertical padding between nodes
    /// * `position_starting` - The starting position of the nodes
    /// * `graphical_size` - The graphical size of the nodes
    /// * `tree_side` - The side of the tree (-1 for left, 1 for right)
    fn place_tree_positions(
        mut element_tree: Vec<&mut Node>,
        padding_horizontal: f32,
        padding_vertical: f32,
        position_starting: Pos2,
        graphical_size: Size,
        tree_side: f32,
    ) {

        let mut y_cursor = 0.0;
        for first_child in element_tree.iter_mut() {
            let subtree_height = first_child.children_graphical_size.clone().unwrap_or_else(|| first_child.graphical_size.clone().unwrap()).height;
            let center_y = y_cursor + subtree_height / 2.0;

            let first_child_pos = Pos2::new(position_starting.x, center_y);
            Self::place_node_positions(
                first_child,
                first_child_pos,
                graphical_size.clone(),
                tree_side,
                padding_horizontal,
                padding_vertical,
            );
            y_cursor += subtree_height + padding_vertical;
        }
    }

    /// Calculates the position of the parent node based on its children
    /// Arguments
    /// * `children` - A mutable reference to a vector of nodes
    /// # Returns
    /// * `Pos2` - The position of the parent node
    fn position_parent_node(
        children: &mut Vec<Node>,
    ) -> Pos2 {
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for child in children.iter() {
            if let Some(pos) = &child.position_from_initial {
                let size = child.graphical_size.clone().unwrap_or(Size { width: 0.0, height: 0.0 });
                min_y = min_y.min(pos.y - size.height / 2.0);
                max_y = max_y.max(pos.y + size.height / 2.0);
            }
        }

        let center_y = (min_y + max_y) / 2.0;
        Pos2 { x: 0.0, y: center_y }
    }


    /// Places the nodes in the mindmap based on the left-right horizontal layout
    /// # Arguments
    /// * `mindmap` - A mutable reference to the mindmap
    /// # Returns
    /// * `&mut Mindmap` - A mutable reference to the mindmap
    pub fn layout(mindmap: &mut Mindmap) -> &mut Mindmap {
        let data = match mindmap.data.as_mut() {
            Some(data) => data,
            None => return mindmap,
        };

        let children = match data.children.as_mut() {
            Some(children) => children,
            None => &mut vec![],
        };

        let MindmapStyle {
            padding_horizontal,
            padding_vertical,
            ..
        } = mindmap.metadata.style;

        tracing::trace!(
            "Mindmap layout: padding_horizontal: {}, padding_vertical: {}",
            padding_horizontal,
            padding_vertical
        );

        // divide the children into two trees
        let (right_tree,left_tree) = Self::divide_elements_tree(children);

        let position_starting = Pos2::zero();

        Self::place_tree_positions(
            right_tree,
            padding_horizontal,
            padding_vertical,
            position_starting.clone(),
            data.graphical_size.clone().unwrap(),
                1.0
        );

        Self::place_tree_positions(
            left_tree,
            padding_horizontal,
            padding_vertical,
            position_starting.clone(),
            data.graphical_size.clone().unwrap(),
                -1.0
        );

        let parent_position = Self::position_parent_node(children);
        mindmap.data.as_mut().unwrap().position_from_initial = Some(parent_position.clone());

        mindmap

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::Node;
    use crate::layout::size::Size;
    use crate::layout::pos2::Pos2;
    use crate::mindmap::{Mindmap, style::MindmapStyle};
    use crate::mindmap::metadata::MindmapMetadata;

    fn create_node(width: f32, height: f32) -> Node {
        Node {
            graphical_size: Some(Size { width, height }),
            children_graphical_size: Some(Size { width, height }),
            children: Some(vec![]),
            position_from_initial: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_place_node_no_children() {
        let mut node = create_node( 100.0, 50.0);
        let parent_pos = Pos2::zero();
        let parent_size = Size { width: 50.0, height: 50.0 };

        LeftRightHorizontalLayout::place_node_positions(
            &mut node,
            parent_pos,
            parent_size,
            1.0,
            20.0,
            10.0,
        );

        let pos = node.position_from_initial.unwrap();
        assert_eq!(pos.x, 95.0);
        assert_eq!(pos.y, 0.0);
    }

    #[test]
    fn test_place_node_with_children() {
        let child1 = create_node( 50.0, 20.0);
        let child2 = create_node( 50.0, 20.0);
        let mut node = create_node( 100.0, 50.0);
        node.children = Some(vec![child1, child2]);
        node.children_graphical_size = Some(Size { width: 100.0, height: 60.0 });

        let parent_pos = Pos2::zero();
        let parent_size = Size { width: 50.0, height: 50.0 };

        LeftRightHorizontalLayout::place_node_positions(
            &mut node,
            parent_pos,
            parent_size,
            1.0,
            10.0,
            10.0,
        );

        let children = node.children.unwrap();
        assert_eq!(children.len(), 2);
        assert!(children[0].position_from_initial.is_some());
        assert!(children[1].position_from_initial.is_some());
        assert_eq!(children[0].position_from_initial.clone().unwrap().x, 170.0);
        assert_eq!(children[0].position_from_initial.clone().unwrap().y, -20.0);
        assert_eq!(children[1].position_from_initial.clone().unwrap().x, 170.0);
        assert_eq!(children[1].position_from_initial.clone().unwrap().y, 10.0);
    }

    #[test]
    fn test_position_parent_node_centered() {
        let mut child1 = create_node( 20.0, 10.0);
        child1.position_from_initial = Some(Pos2::new(0.0, -40.0));

        let mut child2 = create_node( 20.0, 10.0);
        child2.position_from_initial = Some(Pos2::new(0.0, 30.0));

        let mut children = vec![child1, child2];
        let pos = LeftRightHorizontalLayout::position_parent_node(&mut children);

        assert_eq!(pos.x, 0.0);
        assert_eq!(pos.y, -5.0);
    }

    #[test]
    fn test_layout_mindmap() {
        let child1 = create_node( 40.0, 20.0);
        let child2 = create_node( 40.0, 20.0);
        let mut data = create_node( 60.0, 40.0);
        data.children = Some(vec![child1, child2]);
        data.children_graphical_size = Some(Size { width: 100.0, height: 60.0 });

        let mut mindmap = Mindmap {
            data: Some(data),
            metadata: MindmapMetadata {
                style: MindmapStyle {
                    padding_horizontal: 10.0,
                    padding_vertical: 5.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        let result = LeftRightHorizontalLayout::layout(&mut mindmap);
        assert!(result.data.as_ref().unwrap().position_from_initial.is_some());
        assert_eq!(result.data.as_ref().unwrap().position_from_initial.clone().unwrap().x, 0.0);
        assert_eq!(result.data.as_ref().unwrap().position_from_initial.clone().unwrap().y, 10.0);
    }
}
