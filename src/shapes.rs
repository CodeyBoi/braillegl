use std::{collections::HashMap, f32::consts::PI, fs::File, io::{BufRead, BufReader}, path::Path, slice::Iter};

use crate::{math::Vec3f, vertex::{Vertex, VertexArray}};

pub struct Shape {
    va: VertexArray,
    triangles: Vec<(usize, usize, usize)>,
}

impl Shape {
    pub fn new(
        positions: Vec<Vec3f>, 
        normals: Vec<Vec3f>,
        texcoords: Vec<(f32, f32)>,
        triangles: Vec<(usize, usize, usize)>,
    ) -> Self {
        let mut va = VertexArray::with_capacity(positions.len());
        for ((position, normal), texcoord) in positions.iter()
            .zip(&normals)
            .zip(&texcoords)
        {
            va.push(Vertex::new(*position, *normal, *texcoord));
        }
        Self { va, triangles }
    }

    pub fn with_tris(
        positions: Vec<Vec3f>, 
        triangles: Vec<(usize, usize, usize)>,
    ) -> Self {
        let normals = Self::gen_normals(&positions, &triangles);
        Self::with_normals(positions, normals, triangles)
    }

    pub fn with_texcoords(
        positions: Vec<Vec3f>, 
        triangles: Vec<(usize, usize, usize)>,
        texcoords: Vec<(f32, f32)>,
    ) -> Self {
        let normals = Self::gen_normals(&positions, &triangles);
        Self::new(positions, normals, texcoords, triangles)
    }

    pub fn with_normals(
        positions: Vec<Vec3f>, 
        normals: Vec<Vec3f>,
        triangles: Vec<(usize, usize, usize)>,
    ) -> Self {
        let mut va = VertexArray::with_capacity(positions.len());
        for (position, normal) in positions.iter().zip(&normals) {
            va.push(Vertex::with_pos_normal(*position, *normal));
        }
        Self { va, triangles }
    }

    fn gen_normals(
        positions: &Vec<Vec3f>, 
        triangles: &Vec<(usize, usize, usize)>,
    ) -> Vec<Vec3f> {
        let mut normals = vec![Vec3f::zero(); positions.len()];
        for (i0, i1, i2) in triangles {
            let p0 = positions[*i0];
            let p1 = positions[*i1];
            let p2 = positions[*i2];
            // Cross product (p1 - p0)x(p2 - p0) gives the normal for the triangle
            // with points p0, p1 and p2
            let tri_normal = (p1 - p0).cross(&(p2 - p0)).normalize();

            normals[*i0] += tri_normal;
            normals[*i1] += tri_normal;
            normals[*i2] += tri_normal;
        }
        for normal in &mut normals {
            *normal = normal.normalize();
        }
        normals
    }

    pub fn triangles(&self) -> Iter<(usize, usize, usize)> {
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

    let vertex_count = (longitude_points * latitude_points) as usize;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut texcoords = Vec::with_capacity(vertex_count);
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
            let tx = j as f32 / (latitude_points - 1) as f32;
            let ty = i as f32 / (longitude_points - 1) as f32;
            positions.push(position);
            texcoords.push((tx, ty));
        }
    }

    let no_of_triangles = 2 * ((latitude_points - 1) * (longitude_points - 1)) as usize;
    let mut triangles = Vec::with_capacity(no_of_triangles);
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
            triangles.push((idxs[0], idxs[1], idxs[2]));
            triangles.push((idxs[3], idxs[4], idxs[5]));
        }
    }
    Shape::with_texcoords(positions, triangles, texcoords)
}

pub fn make_icosphere(radius: f32, refinement_depth: u8) -> Shape {

    assert!(radius > 0.0);

    let vertex_count = 12 * 2_usize.pow(refinement_depth as u32);
    let mut positions = Vec::with_capacity(vertex_count);

    let s = ((5.0 - 5.0_f32.sqrt()) / 10.0).sqrt() * radius;
    let t = ((5.0 + 5.0_f32.sqrt()) / 10.0).sqrt() * radius;

    positions.push(Vec3f::new(-s, t, 0.0));
    positions.push(Vec3f::new(s, t, 0.0));
    positions.push(Vec3f::new(-s, -t, 0.0));
    positions.push(Vec3f::new(s, -t, 0.0));

    positions.push(Vec3f::new(0.0, -s, t));
    positions.push(Vec3f::new(0.0, s, t));
    positions.push(Vec3f::new(0.0, -s, -t));
    positions.push(Vec3f::new(0.0, s, -t));

    positions.push(Vec3f::new(t, 0.0, -s));
    positions.push(Vec3f::new(t, 0.0, s));
    positions.push(Vec3f::new(-t, 0.0, -s));
    positions.push(Vec3f::new(-t, 0.0, s));

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
            let a = find_middle_point(i.0, i.1, radius, &mut positions, &mut cache);
            let b = find_middle_point(i.1, i.2, radius, &mut positions, &mut cache);
            let c = find_middle_point(i.2, i.0, radius, &mut positions, &mut cache);

            new_triangles.push((i.0, a, c));
            new_triangles.push((i.1, b, a));
            new_triangles.push((i.2, c, b));
            new_triangles.push((a, b, c));
        }
        triangles = new_triangles;
    }

    fn find_middle_point(
        a: usize, b: usize, radius: f32,
        positions: &mut Vec<Vec3f>, 
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
            let (p0, p1) = (positions[a], positions[b]);
            positions.push((p0 + p1).normalize().scale(radius));
            let index = positions.len() - 1;
            cache.insert((a, b), index);
            index
        }
    }
    Shape::with_tris(positions, triangles)
}

// pub fn make_cuboid(width: f32, height: f32, length: f32, splits: u64) {
//     let x0 = -width  / 2.0;
//     let y0 = -height / 2.0;
//     let z0 = -length / 2.0;
//     let point_distance = [
//             FloatOrd{ 0: width }, 
//             FloatOrd{ 0: height }, 
//             FloatOrd{ 0: length }
//         ].iter().max().unwrap().0 / splits as f32;

//     let x_steps = (width  / point_distance) as usize;
//     let y_steps = (height / point_distance) as usize;
//     let z_steps = (length / point_distance) as usize;

//     for i in 0..x_steps {

//     }
// }

pub fn make_quad(width: f32, length: f32, splits: u64) -> Shape {

    assert!(width > 0.0);
    assert!(length > 0.0);

    let x0 = -width  / 2.0;
    let z0 = -length / 2.0;
    let point_difference = width.max(length) / (splits + 1) as f32;
    let x_points = (width  / point_difference) as usize + 1;
    let z_points = (length / point_difference) as usize + 1;

    let no_of_vertices = x_points * z_points;
    let mut positions = Vec::with_capacity(no_of_vertices);
    let mut texcoords = Vec::with_capacity(no_of_vertices);

    for i in 0..x_points {
        for j in 0..z_points {
            let x = x0 + i as f32 * width  / (x_points - 1) as f32;
            let z = z0 + j as f32 * length / (x_points - 1) as f32;
            
            positions.push(Vec3f::new(x, 0.0, z));
            texcoords.push((i as f32 / (x_points - 1) as f32, j as f32 / (z_points - 1) as f32));
        }
    }

    let no_of_triangles = 2 * ((x_points - 1) * (z_points - 1)) as usize;
    let mut triangles = Vec::with_capacity(no_of_triangles);
    for col in 0..x_points - 1 {
        for row in 0..z_points - 1 {
            // The current indices to be appended
            let idxs = [
                ((col + 0) + (row + 0) * x_points) as usize,
                ((col + 1) + (row + 0) * x_points) as usize,
                ((col + 1) + (row + 1) * x_points) as usize,
                ((col + 0) + (row + 0) * x_points) as usize,
                ((col + 1) + (row + 1) * x_points) as usize,
                ((col + 0) + (row + 1) * x_points) as usize,
            ];
            triangles.push((idxs[0], idxs[1], idxs[2]));
            triangles.push((idxs[3], idxs[4], idxs[5]));
        }
    }
    Shape::with_texcoords(positions, triangles, texcoords)    
}

pub fn load_from_file<P: AsRef<Path>>(filepath: P) -> Shape {

    let mut triangles = Vec::new();
    let mut texcoord_vecs = Vec::new();
    let mut positions = Vec::new();
    let mut texcoords = Vec::new();

    let reader = BufReader::new(File::open(filepath).unwrap());
    for line in reader.lines() {
        let line = line.unwrap();
        if line.starts_with("v ") {
            let p = line[2..].split_whitespace().map(|x|
                x.parse().unwrap()
            ).collect::<Vec<f32>>();
            positions.push(Vec3f::new(p[0], p[1], p[2]));
        } else if line.starts_with("vt ") {
            let p = line[3..].split_whitespace().map(|x|
                x.parse().unwrap()
            ).collect::<Vec<f32>>();
            texcoord_vecs.push((p[0], p[1]));

        } else if line.starts_with("f ") {
            let mut tri = Vec::with_capacity(4);
            line[1..].split_whitespace().map(|str| {
                str.split("/").take(2).enumerate().map(|(i, val)| {
                    let val: i32 = val.parse().unwrap();
                    let val = if val >= 0 {
                        val as usize - 1
                    } else {
                        if i == 0 {
                            (positions.len() as i32 + val) as usize
                        } else {
                            (texcoord_vecs.len() as i32 + val) as usize
                        }
                    };
                    if i == 0 {
                        tri.push(val);
                    } else {
                        texcoords.push(texcoord_vecs[val]);
                    }
                }).last();
            }).last();
            let i0 = tri[0];
            for (i1, i2) in tri[1..].iter().zip(&tri[2..]) {
                triangles.push((i0, *i1, *i2));
            }
        }
    }
    let normals = Shape::gen_normals(&positions, &triangles);
    println!("{} {} {}", positions.len(), normals.len(), texcoords.len());
    if texcoords.is_empty() {
        Shape::with_normals(positions, normals, triangles)
    } else {
        Shape::new(positions, normals, texcoords, triangles)
    }
}