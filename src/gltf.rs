use gltf::camera::Projection;
use gltf::scene::iter;
use gltf::{Node, Scene};

use crate::object;
use crate::object::Transform;

pub fn extract<T: Clone>(scene: &Scene, parse_fn: fn(&Node, Transform) -> Option<T>) -> Option<T> {
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());

        let maybe_object = parse_fn(&node, carry);
        if maybe_object.is_some() {
            return maybe_object;
        }

        let objects = visit_nodes(node.children(), carry, parse_fn, true);
        if !objects.is_empty() {
            return objects.first().cloned();
        }
    }

    None
}

pub fn extract_all<T>(scene: &Scene, parse_fn: fn(&Node, Transform) -> Option<T>) -> Vec<T> {
    let mut result = vec![];
    for node in scene.nodes() {
        let carry = object::Transform::from(node.transform());

        let maybe_object = parse_fn(&node, carry);
        if let Some(object) = maybe_object {
            result.push(object);
        }

        let objects = visit_nodes(node.children(), carry, parse_fn, false);

        if !objects.is_empty() {
            result.extend(objects);
        }
    }

    result
}

pub fn get_camera(node: &Node, carry: Transform) -> Option<object::Camera> {
    if let Some(camera) = node.camera() {
        if let Projection::Perspective(perspective) = camera.projection() {
            return Some(object::Camera {
                parent_transform: carry,
                transform: object::Transform::from(node.transform()),
                aspect_ratio: perspective.aspect_ratio().unwrap_or(1.0),
                yfov: perspective.yfov(),
                zfar: perspective.zfar().unwrap_or(100.0),
                znear: perspective.znear(),
            });
        }
    }
    None
}

pub fn get_mesh(node: &Node, carry: Transform) -> Option<object::Mesh> {
    if node.mesh().is_some() {
        return Some(object::Mesh {
            parent_transform: carry,
            transform: object::Transform::from(node.transform()),
        });
    }
    None
}

fn visit_nodes<T>(
    nodes: iter::Children,
    carry: Transform,
    parse_fn: fn(&Node, Transform) -> Option<T>,
    break_on_first: bool,
) -> Vec<T> {
    let mut result = vec![];
    for node in nodes {
        if let Some(mesh) = parse_fn(&node, carry) {
            result.push(mesh);
            if break_on_first {
                return result;
            }
        }

        let carry = carry * object::Transform::from(node.transform());

        let objects = visit_nodes(node.children(), carry, parse_fn, break_on_first);
        if !objects.is_empty() {
            result.extend(objects);
            if break_on_first {
                return result;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};

    use super::*;

    #[test]
    fn camera() -> Result<()> {
        let gltf = load_test_model("testdata/iphone.gltf")?;
        let doc = gltf.document;
        let scene = doc.default_scene().ok_or(anyhow!("no default scene"))?;
        let camera = extract(&scene, get_camera);
        // todo compare properties
        assert!(camera.is_some());
        Ok(())
    }

    #[test]
    fn meshes() -> Result<()> {
        let gltf = load_test_model("testdata/duvet-cover.gltf")?;
        let doc = gltf.document;
        let scene = doc.default_scene().ok_or(anyhow!("no default scene"))?;
        let meshes = extract_all(&scene, get_mesh);
        assert_eq!(meshes.len(), 3);
        Ok(())
    }

    fn load_test_model(path: &str) -> Result<gltf::Gltf> {
        let content = std::fs::read(path)?;
        Ok(gltf::Gltf::from_slice(content.as_slice())?)
    }
}
