use std::{f32::consts::PI, fs::{File}, io::{BufRead, BufReader}, path::Path, slice::Iter};

use crate::{math::Vec3f, vertex::{Vertex, VertexArray}};

pub struct Shape {
    va: VertexArray,
    indices: Vec<(usize, usize, usize)>,
    pub normals: Vec<Vec3f>,
}

impl Shape {
    pub fn new(vertex_array: VertexArray, indices: Vec<(usize, usize, usize)>) -> Self {
        // Cross product (p1 - p0)x(p2 - p0) gives the normal for the triangle
        // with points p0, p1 and p2
        let mut normals = Vec::with_capacity(indices.len());
        for (i0, i1, i2) in &indices {
            let p0 = vertex_array[*i0].position;
            let p1 = vertex_array[*i1].position;
            let p2 = vertex_array[*i2].position;
            let normal = (p1 - p0).cross(&(p2 - p0)).normalize();
            normals.push(normal);
        }
        Shape { va: vertex_array, indices, normals }
    }

    pub fn indices(&self) -> Iter<(usize, usize, usize)> {
        self.indices.iter()
    }

    pub fn get(&self, index: usize) -> &Vertex {
        &self.va[index]
    }
}

pub fn make_sphere(
    radius: f32, 
    longitude_splits: u64, 
    latitude_splits: u64
) -> Shape {
    
    assert!(radius > 0.0);
    assert!(longitude_splits >= 2);
    assert!(latitude_splits >= 1);

    let longitude_points = longitude_splits + 2;
    let latitude_points = latitude_splits + 2;

    let mut va = VertexArray::new((longitude_points * latitude_points) as usize);
    for i in 0..longitude_points {
        for j in 0..latitude_points {
            let phi = PI * i as f32 / (longitude_points - 1) as f32;
            let theta = 2.0 * PI * j as f32 / (latitude_points - 1) as f32;
            let (sinphi, cosphi) = phi.sin_cos();
            let (sintheta, costheta) = theta.sin_cos();
            
            let position = Vec3f::new(
                radius * sintheta * sinphi,
                -radius * cosphi,
                radius * costheta * sinphi,
            );
            va.push(Vertex::new(position));
        }
    }

    let no_of_triangles = 2 * ((latitude_points - 1) * (longitude_points - 1)) as usize;
    let mut indices = Vec::with_capacity(no_of_triangles);
    for col in 0..longitude_points - 1 {
        for row in 0..latitude_points - 1 {
            // The current indices to be appended
            let idxs = [
                ((col + 0) + (row + 0) * longitude_points) as usize,
                ((col + 1) + (row + 0) * longitude_points) as usize,
                ((col + 1) + (row + 1) * longitude_points) as usize,
                ((col + 0) + (row + 0) * longitude_points) as usize,
                ((col + 1) + (row + 1) * longitude_points) as usize,
                ((col + 0) + (row + 1) * longitude_points) as usize,
            ];
            indices.push((idxs[0], idxs[1], idxs[2]));
            indices.push((idxs[3], idxs[4], idxs[5]));
        }
    }
    Shape::new(va, indices)
}

pub fn load_from_file<P: AsRef<Path>>(filepath: P) -> Shape {
    let mut va = VertexArray::new(512);
    let mut indices = Vec::with_capacity(1536);
    let reader = BufReader::new(File::open(filepath).unwrap());
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("v ") {
            let p = line[2..].split_whitespace().map(|x|
                x.parse().unwrap()
            ).collect::<Vec<f32>>();
            va.push(Vertex::new(Vec3f::new(p[0], p[1], p[2])));
        } else if line.starts_with("f ") {
            let idxs = line[2..].split_whitespace()
                .map(|x|
                    if x.contains("/") {
                        let val = x.split_once("/").unwrap().0.parse::<i32>().unwrap();
                        if val < 0 {
                            (va.len() as i32 + val) as usize
                        } else {
                            val as usize - 1
                        }
                    } else {
                        let val = x.parse::<i32>().unwrap();
                        if val < 0 {
                            (va.len() as i32 + val) as usize
                        } else {
                            val as usize - 1
                        }
                    }
                ).collect::<Vec<usize>>();
            indices.push((idxs[0], idxs[1], idxs[2]));
            if idxs.len() == 4 {
                indices.push((idxs[0], idxs[2], idxs[3]));
            }
        }
    }
    Shape::new(va, indices)
}