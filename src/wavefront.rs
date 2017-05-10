use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
 
use define::{V, Vn, Vt, Vtn, Vtnt, VertexSlice};
use gfx;

use cgmath::prelude::*;
use cgmath::{Point2, Point3,Vector3};


#[derive(Clone, Debug)] 
pub struct WavefrontMesh<V: WavefrontVertex> { 
    pub indicies: Vec<u32>, 
    pub verts: Vec<V>, 
}

impl<V: WavefrontVertex> WavefrontMesh<V>
    where V: gfx::traits::Pod + gfx::pso::buffer::Structure<gfx::format::Format> 
{
    fn create_buffer<R, F>(&self, factory: &mut F) -> VertexSlice<R, V> 
        where R: gfx::Resources, F: gfx::traits::FactoryExt<R>
    {
        factory.create_vertex_buffer_with_slice(&self.verts, &self.indicies[..])
    }
}
 
pub trait WavefrontVertex { 
    fn load(inds: Inds, pos: &[[f32; 3]], tex: &[[f32; 2]], nor: &[[f32; 3]]) -> Self;

    fn requires_edges() -> bool;

    fn acc_tri(a: &mut Self, b: &mut Self, c: &mut Self);
} 
 
impl WavefrontVertex for V { 
    fn load(inds: Inds, pos: &[[f32; 3]], _: &[[f32; 2]], _: &[[f32; 3]]) -> Self { 
        V { a_pos: pos[inds.0] } 
    }

    fn requires_edges() -> bool { false }
    fn acc_tri(_: &mut Self, _: &mut Self, _: &mut Self) {
        unreachable!();
    }
} 
 
impl WavefrontVertex for Vn { 
    fn load(inds: Inds, pos: &[[f32; 3]], _: &[[f32; 2]], nor: &[[f32; 3]]) -> Self { 
        Vn { 
            a_pos: pos[inds.0], 
            a_nor: inds.2.map(|i| nor[i]).unwrap_or([0f32; 3]), 
        } 
    }

    fn requires_edges() -> bool { false }
    fn acc_tri(_: &mut Self, _: &mut Self, _: &mut Self) {
        unreachable!();
    }
} 
 
impl WavefrontVertex for Vt { 
    fn load(inds: Inds, pos: &[[f32; 3]], tex: &[[f32; 2]], _: &[[f32; 3]]) -> Self { 
        Vt { 
            a_pos: pos[inds.0], 
            a_tex: inds.1.map(|i| tex[i]).unwrap_or([0f32; 2]), 
        } 
    }

    fn requires_edges() -> bool { false }
    fn acc_tri(_: &mut Self, _: &mut Self, _: &mut Self) {
        unreachable!();
    }
} 
 
impl WavefrontVertex for Vtn { 
    fn load(inds: Inds, pos: &[[f32; 3]], tex: &[[f32; 2]], nor: &[[f32; 3]]) -> Self { 
        Vtn { 
            a_pos: pos[inds.0], 
            a_tex: inds.1.map(|i| tex[i]).unwrap_or([0f32; 2]), 
            a_nor: inds.2.map(|i| nor[i]).unwrap_or([0f32; 3]), 
        } 
    }

    fn requires_edges() -> bool { false }
    fn acc_tri(_: &mut Self, _: &mut Self, _: &mut Self) {
        unreachable!();
    }
}

fn add_to<T: ::std::ops::AddAssign + Copy>(to: &mut [T; 3], vec: &Vector3<T>) {
    to[0] += vec.x;
    to[1] += vec.y;
    to[2] += vec.z;
}

impl WavefrontVertex for Vtnt {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        tex: &[[f32; 2]], 
        nor: &[[f32; 3]]) -> Self 
    {
        Vtnt {
            a_pos: pos[inds.0],
            a_tex: inds.1.map(|i| tex[i]).unwrap_or([0f32; 2]),
            a_nor: inds.2.map(|i| nor[i]).unwrap_or([0f32; 3]),
            a_tan: [0.; 3],
            a_btn: [0.; 3],
        }
    }

    fn requires_edges() -> bool { true }

    fn acc_tri(a: &mut Self, b: &mut Self, c: &mut Self) {
        // positions
        let pos1 = Point3::from(a.a_pos);
        let pos2 = Point3::from(b.a_pos);
        let pos3 = Point3::from(c.a_pos);
        // texture coordinates
        let uv1 = Point2::from(a.a_tex);
        let uv2 = Point2::from(b.a_tex);
        let uv3 = Point2::from(c.a_tex);

        // deltas
        let edge1 = pos2 - pos1;
        let edge2 = pos3 - pos1;
        let delta_uv1 = uv2 - uv1;
        let delta_uv2 = uv3 - uv1;

        let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

        let tan = Vector3::new(
            f * (delta_uv2.y * edge1.x - delta_uv1.y * edge2.x),
            f * (delta_uv2.y * edge1.y - delta_uv1.y * edge2.y),
            f * (delta_uv2.y * edge1.z - delta_uv1.y * edge2.z),
        ).normalize();

        let bitan = Vector3::new(
            f * (-delta_uv2.x * edge1.x + delta_uv1.x * edge2.x),
            f * (-delta_uv2.x * edge1.y + delta_uv1.x * edge2.y),
            f * (-delta_uv2.x * edge1.z + delta_uv1.x * edge2.z),
        ).normalize();

        add_to(&mut a.a_tan, &tan);
        add_to(&mut b.a_tan, &tan);
        add_to(&mut c.a_tan, &tan);

        add_to(&mut a.a_btn, &bitan);
        add_to(&mut b.a_btn, &bitan);
        add_to(&mut c.a_btn, &bitan);
    }
}
 
pub type Inds = (usize, Option<usize>, Option<usize>); 
 
// 
// The accual parsing stuff, everything below here is pretty nasty 
// 
 
type IInds = (isize, Option<isize>, Option<isize>); 
 
fn renorm_ind(x: isize, len: usize) -> Result<usize, ()> { 
    let len = len as isize; 
    match x { 
        _ if x > 0 && x <= len => Ok((x - 1) as usize), 
        _ if x < 0 && -x <= len => Ok((len + x) as usize), 
        _ => Err(()), 
    } 
} 
 
fn add_index<V: WavefrontVertex>(inds: IInds, 
                                 verts: &mut Vec<V>, 
                                 dedup: &mut HashMap<Inds, usize>, 
                                 pos: &[[f32; 3]], 
                                 tex: &[[f32; 2]], 
                                 nor: &[[f32; 3]], 
                                 line: usize) 
                                 -> Result<u32, usize> { 
    let err = Err(line); 
 
    let a = renorm_ind(inds.0, pos.len()).or(err)?; 
    let b = match inds.1 { 
        Some(x) => Some(renorm_ind(x, tex.len()).or(err)?), 
        None => None, 
    }; 
    let c = match inds.2 { 
        Some(x) => Some(renorm_ind(x, nor.len()).or(err)?), 
        None => None, 
    }; 
 
    let inds = (a, b, c); 
 
 
    Ok(*dedup 
            .entry(inds) 
            .or_insert_with(|| { 
                                verts.push(V::load(inds, pos, tex, nor)); 
                                verts.len() - 1 
                            }) as u32) 
} 
 
pub fn parse_inds(data: &str, line: usize) -> Result<IInds, usize> { 
    let mut inds = data.split('/'); 
    let err = Err(line); 
 
    let pos = inds.next().ok_or(line)?.parse().or(err)?; 
    let tex = match inds.next() { 
        Some("") | None => None, 
        Some(v) => Some(v.parse().or(err)?), 
    }; 
    let norm = match inds.next() { 
        Some("") | None => None, 
        Some(v) => Some(v.parse().or(err)?), 
    }; 
 
    Ok((pos, tex, norm)) 
}

pub fn load_obj<V: WavefrontVertex, R: BufRead>(read: R) -> Result<WavefrontMesh<V>, usize> {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut nor = Vec::new();
    let mut tex = Vec::new();

    let mut dedup = HashMap::new();
    let mut verts = Vec::new();

    let mut inds = Vec::new();

    let mut linen = 0usize;

    for line in read.lines() {
        linen += 1;

        let line = line.or(Err(linen))?;
        let mut line = line.split_whitespace();

        let err = Err(linen);

        match line.next() {
            Some("v") => {
                pos.push([
                    line.next().ok_or(linen)?.parse().or(err)?,
                    line.next().ok_or(linen)?.parse().or(err)?,
                    line.next().ok_or(linen)?.parse().or(err)?,
                ])
            },
            Some("vt") => {
                tex.push([
                    line.next().ok_or(linen)?.parse().or(err)?,
                    line.next().ok_or(linen)?.parse().or(err)?,
                ])
            },
            Some("vn") => {
                nor.push([
                    line.next().ok_or(linen)?.parse().or(err)?,
                    line.next().ok_or(linen)?.parse().or(err)?,
                    line.next().ok_or(linen)?.parse().or(err)?,
                ])
            },
            Some("f") => {
                let mut triv = Vec::with_capacity(3);
                for d in line {
                    triv.push(add_index(
                        parse_inds(d, linen)?, 
                        &mut verts, 
                        &mut dedup, 
                        &pos[..], 
                        &tex[..], 
                        &nor[..],
                        linen
                    )?)
                };

                if V::requires_edges() {
                    if triv.len() != 3 { return Err(linen) }
                    let (a, b, c) = (triv[0], triv[1], triv[2]);
                    
                    if a == b || b == c || c == a { return Err(linen); } // indicies must be different
                    let tri = (&verts[a as usize], &verts[b as usize], &verts[c as usize]);

                    let tri = unsafe {
                        (&mut *(tri.0 as *const V as *mut V),
                         &mut *(tri.1 as *const V as *mut V),
                         &mut *(tri.2 as *const V as *mut V))
                    };

                    V::acc_tri(tri.0, tri.1, tri.2);
                }

                inds.append(&mut triv);
            },
            Some(_) => (),
            None => (),
        }
    }

    Ok(WavefrontMesh {
        verts: verts,
        indicies: inds,

    })
}

pub fn open_obj<V: WavefrontVertex, F, R, P: AsRef<Path>>(path: P, factory: &mut F) 
-> Result<VertexSlice<R, V>, String>
    where V: gfx::traits::Pod + gfx::pso::buffer::Structure<gfx::format::Format>,
    R: gfx::Resources, 
    F: gfx::traits::FactoryExt<R>
{
    let display = format!("{}", path.as_ref().display());

    match File::open(path) {
        Ok(f) => match load_obj(BufReader::new(f)) {
            Ok(obj) => Ok(obj.create_buffer(factory)),
            Err(line) => Err(format!("Error parsing \"{}\" on line #{}", display, line)),
        },
        Err(e) => Err(format!("File \"{}\" could not be opened: {:?}", display, e)),
    }
}