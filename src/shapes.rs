use std::{collections::HashMap, f32::consts::PI, fs::{File}, io::{BufRead, BufReader}, path::Path, slice::Iter};

use crate::{math::Vec3f, vertex::{Vertex, VertexArray}};

pub struct Shape {
    va: VertexArray,
    triangles: Vec<(usize, usize, usize)>,
    pub normals: Vec<Vec3f>,
}

impl Shape {
    pub fn new(
        va: VertexArray, 
        triangles: Vec<(usize, usize, usize)>,
        normals: Vec<Vec3f>,
    ) -> Self {
        Self { va, triangles, normals }
    }
    pub fn with_tris(va: VertexArray, triangles: Vec<(usize, usize, usize)>) -> Self {
        let normals = Self::gen_normals(&va, &triangles);
        Self::new(va, triangles, normals)
    }

    fn gen_normals(
        va: &VertexArray, 
        triangles: &Vec<(usize, usize, usize)>,
    ) -> Vec<Vec3f> {
        let mut normals = Vec::with_capacity(triangles.len());
        for (i0, i1, i2) in triangles {
            let p0 = va[*i0].position;
            let p1 = va[*i1].position;
            let p2 = va[*i2].position;
            // Cross product (p1 - p0)x(p2 - p0) gives the normal for the triangle
            // with points p0, p1 and p2
            let normal = (p1 - p0).cross(&(p2 - p0)).normalize();
            normals.push(normal);
        }
        normals
    }

    pub fn indices(&self) -> Iter<(usize, usize, usize)> {
        self.triangles.iter()
    }

    pub fn get(&self, index: usize) -> &Vertex {
        &self.va[index]
    }
}

pub fn make_uv_sphere(
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
    Shape::with_tris(va, indices)
}

pub fn make_icosphere(radius: f32, refinement_depth: u8) -> Shape {
    let mut va = VertexArray::new(12 * 2_usize.pow(refinement_depth as u32));

    let s = ((5.0 - 5.0_f32.sqrt()) / 10.0).sqrt() * radius;
    let t = ((5.0 + 5.0_f32.sqrt()) / 10.0).sqrt() * radius;

    va.push(Vertex::with_pos(Vec3f::new(-s, t, 0.0)));
    va.push(Vertex::with_pos(Vec3f::new(s, t, 0.0)));
    va.push(Vertex::with_pos(Vec3f::new(-s, -t, 0.0)));
    va.push(Vertex::with_pos(Vec3f::new(s, -t, 0.0)));

    va.push(Vertex::with_pos(Vec3f::new(0.0, -s, t)));
    va.push(Vertex::with_pos(Vec3f::new(0.0, s, t)));
    va.push(Vertex::with_pos(Vec3f::new(0.0, -s, -t)));
    va.push(Vertex::with_pos(Vec3f::new(0.0, s, -t)));

    va.push(Vertex::with_pos(Vec3f::new(t, 0.0, -s)));
    va.push(Vertex::with_pos(Vec3f::new(t, 0.0, s)));
    va.push(Vertex::with_pos(Vec3f::new(-t, 0.0, -s)));
    va.push(Vertex::with_pos(Vec3f::new(-t, 0.0, s)));

    let mut triangles = Vec::new();

    triangles.push((0, 11, 5));
    triangles.push((0, 5, 1));
    triangles.push((0, 1, 7));
    triangles.push((0, 7, 10));
    triangles.push((0, 10, 11));

    triangles.push((1, 5, 9));
    triangles.push((5, 11, 4));
    triangles.push((11, 10, 2));
    triangles.push((10, 7, 6));
    triangles.push((7, 1, 8));

    triangles.push((3, 9, 4));
    triangles.push((3, 4, 2));
    triangles.push((3, 2, 6));
    triangles.push((3, 6, 8));
    triangles.push((3, 8, 9));

    triangles.push((4, 9, 5));
    triangles.push((2, 4, 11));
    triangles.push((6, 2, 10));
    triangles.push((8, 6, 7));
    triangles.push((9, 8, 1));

    for _ in 0..refinement_depth {
        let mut new_triangles = Vec::new();
        let mut cache = HashMap::new();
        for i in &triangles {        
            let a = find_middle_point(i.0, i.1, radius, &mut va, &mut cache);
            let b = find_middle_point(i.1, i.2, radius, &mut va, &mut cache);
            let c = find_middle_point(i.2, i.0, radius, &mut va, &mut cache);

            new_triangles.push((i.0, a, c));
            new_triangles.push((i.1, b, a));
            new_triangles.push((i.2, c, b));
            new_triangles.push((a, b, c));
        }
        triangles = new_triangles;
    }

    fn find_middle_point(
        a: usize, b: usize, radius: f32,
        va: &mut VertexArray, 
        cache: &mut HashMap<(usize, usize), usize>
    ) -> usize {
        let (a, b) = if a < b {
            (a, b)
        } else {
            (b, a)
        };
        if cache.contains_key(&(a, b)) {
            *cache.get(&(a, b)).unwrap()
        } else {
            let (p0, p1) = (va[a].position, va[b].position);
            va.push(Vertex::with_pos((p0 + p1).normalize().scale(radius)));
            let index = va.len() - 1;
            cache.insert((a, b), index);
            index
        }
    }
    Shape::with_tris(va, triangles)
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
        } else if line.starts_with("vt ")
    }
    Shape::with_tris(va, indices)
}