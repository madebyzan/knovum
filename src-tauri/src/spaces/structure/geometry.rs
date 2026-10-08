use std::collections::HashMap;

use super::appearance::{validate_content_appearance, validate_section_appearance};
use super::contracts::{LocalTransform, Point, Size, StructureLayout, StructureNode};

const MAX_COORDINATE: f64 = 1_000_000_000.0;
const MIN_SCALE: f64 = 0.000_001;
const MAX_SCALE: f64 = 1_000_000.0;

pub(super) fn validate_layout(layout: &StructureLayout) -> Result<(), String> {
    match layout {
        StructureLayout::Content {
            transform,
            size,
            appearance,
            file_subpath,
            ..
        } => {
            validate_transform(transform)?;
            validate_size(size)?;
            validate_content_appearance(appearance)?;
            if file_subpath
                .as_ref()
                .is_some_and(|value| !value.starts_with('#'))
            {
                return Err("File subpath must start with #".into());
            }
        }
        StructureLayout::Group { transform, .. } => validate_transform(transform)?,
        StructureLayout::Section {
            position,
            size,
            appearance,
            ..
        } => {
            validate_point(position)?;
            validate_size(size)?;
            if size.width < 240.0 || size.height < 160.0 {
                return Err("Section size must be at least 240×160".into());
            }
            validate_section_appearance(appearance)?;
        }
    }
    Ok(())
}

fn validate_point(point: &Point) -> Result<(), String> {
    if valid_coordinate(point.x) && valid_coordinate(point.y) {
        Ok(())
    } else {
        Err("Invalid structure_node position".into())
    }
}

fn validate_size(size: &Size) -> Result<(), String> {
    if valid_dimension(size.width) && valid_dimension(size.height) {
        Ok(())
    } else {
        Err("Invalid structure_node size".into())
    }
}

fn validate_transform(transform: &LocalTransform) -> Result<(), String> {
    if valid_coordinate(transform.x)
        && valid_coordinate(transform.y)
        && valid_coordinate(transform.rotation)
        && valid_scale(transform.scale)
    {
        Ok(())
    } else {
        Err("Invalid local transform".into())
    }
}

fn valid_coordinate(value: f64) -> bool {
    value.is_finite() && value.abs() <= MAX_COORDINATE
}

fn valid_dimension(value: f64) -> bool {
    value.is_finite() && value > 0.0 && value <= MAX_COORDINATE
}

fn valid_scale(value: f64) -> bool {
    value.is_finite() && (MIN_SCALE..=MAX_SCALE).contains(&value)
}

pub(super) fn validate_world_transform(
    structure_node: &StructureNode,
    structure: &HashMap<&str, &StructureNode>,
    layouts: &HashMap<&str, &StructureLayout>,
) -> Result<(), String> {
    let mut chain = vec![structure_node];
    while let Some(parent_id) = chain.last().and_then(|ancestor| ancestor.parent_id()) {
        chain.push(structure[parent_id]);
    }
    let (mut x, mut y, mut rotation, mut scale) = (0.0, 0.0, 0.0_f64, 1.0);
    for ancestor in chain.into_iter().rev() {
        let (local_x, local_y, local_rotation, local_scale) = match layouts[ancestor.id()] {
            StructureLayout::Content { transform, .. }
            | StructureLayout::Group { transform, .. } => (
                transform.x,
                transform.y,
                transform.rotation,
                transform.scale,
            ),
            StructureLayout::Section { position, .. } => (position.x, position.y, 0.0, 1.0),
        };
        let radians = rotation.to_radians();
        x += scale * (local_x * radians.cos() - local_y * radians.sin());
        y += scale * (local_x * radians.sin() + local_y * radians.cos());
        rotation += local_rotation;
        scale *= local_scale;
        if !valid_coordinate(x)
            || !valid_coordinate(y)
            || !valid_scale(scale)
            || !valid_coordinate(rotation)
        {
            return Err(format!(
                "Invalid accumulated transform for {}",
                structure_node.id()
            ));
        }
    }
    Ok(())
}
