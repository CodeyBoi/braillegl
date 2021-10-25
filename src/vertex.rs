use std::slice::{Iter, IterMut};
use std::ops::{Index, IndexMut};

use crate::math::Vec3f;

#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: Vec3f,
}

impl Vertex {
    pub fn new(position: Vec3f) -> Self {
        Self {
            position,
        }
    }
}

pub struct VertexArray {
    vertices: Vec<Vertex>,
}

impl VertexArray {
    /// Creates a new VertexArray. This is just a `Vec<Vertex>`.
    /// 
    /// # Arguments
    /// `capacity` - the amount of vertices that can be added before a reallocation is needed
    pub fn new(capacity: usize) -> Self {
        Self {
            vertices: Vec::with_capacity(capacity),
        }
    }

    /// Appends a Vertex to the end
    /// 
    /// # Arguments
    /// `vertex` - the vertex to be appended
    pub fn push(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
    }

    /// Returns the length
    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    /// Returns an iterator over the vertices
    pub fn vertices(&self) -> Iter<Vertex> {
        self.vertices.iter()
    }

    /// Returns a mutable iterator over the vertices
    pub fn vertices_mut(&mut self) -> IterMut<Vertex> {
        self.vertices.iter_mut()
    }
}

impl Index<usize> for VertexArray {
    type Output = Vertex;
    fn index(&self, index: usize) -> &Vertex {
        &self.vertices[index]
    }
}

impl IndexMut<usize> for VertexArray {
    fn index_mut(&mut self, index: usize) -> &mut Vertex {
        &mut self.vertices[index]
    }
}