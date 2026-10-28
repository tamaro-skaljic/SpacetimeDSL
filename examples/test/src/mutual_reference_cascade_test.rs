//! Two tables which reference each other. Each table type implements the cascade of the other,
//! and deleting a row of either reaches the rows of the other.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = left_nodes, method(update = true, delete = true))]
#[spacetimedb::table(accessor = left_node)]
pub struct LeftNode {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(LeftNodeId)]
    #[referenced_by(path = crate::mutual_reference_cascade_test, table = right_node)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(RightNodeId)]
    #[foreign_key(
        path = crate::mutual_reference_cascade_test,
        table = right_node,
        column = id,
        on_delete = SetZero
    )]
    pub right_node_id: u64,
}

#[spacetimedsl::dsl(plural_name = right_nodes, method(update = false, delete = true))]
#[spacetimedb::table(accessor = right_node)]
pub struct RightNode {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(RightNodeId)]
    #[referenced_by(path = crate::mutual_reference_cascade_test, table = left_node)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(LeftNodeId)]
    #[foreign_key(
        path = crate::mutual_reference_cascade_test,
        table = left_node,
        column = id,
        on_delete = Delete
    )]
    left_node_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let mut left_node = dsl.create_left_node(CreateLeftNode {
        right_node_id: RightNodeId::new(0),
    })?;
    let right_node = dsl.create_right_node(CreateRightNode {
        left_node_id: left_node.get_id(),
    })?;

    left_node.set_right_node_id(right_node.get_id());
    let left_node = dsl.update_left_node_by_id(left_node)?;

    dsl.delete_right_node_by_id(right_node.get_id())
        .map_err(|error| {
            format!("Deleting a right node should clear the left node referencing it: {error}")
        })?;

    let right_node_id_left = dsl
        .get_left_node_by_id(left_node.get_id())?
        .get_right_node_id()
        .value();
    if right_node_id_left != 0 {
        return Err(format!(
            "Deleting a right node should set the foreign key of the left node referencing it to 0, found {right_node_id_left}!"
        ));
    }

    let other_right_node = dsl.create_right_node(CreateRightNode {
        left_node_id: left_node.get_id(),
    })?;

    dsl.delete_left_node_by_id(left_node.get_id())
        .map_err(|error| {
            format!("Deleting a left node should delete the right nodes referencing it: {error}")
        })?;

    if dsl.get_right_node_by_id(other_right_node.get_id()).is_ok() {
        return Err(
            "Deleting a left node should delete the right nodes referencing it, but one is left!"
                .to_string(),
        );
    }

    Ok(())
}
