use std::io::{BufReader, BufRead};
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;

use glium::vertex::{Vertex};
use glium::{IndexBuffer, VertexBuffer};
use glium::backend::Facade;

#[derive(Clone, Debug)]
pub struct WavefrontMesh<V: WavefrontVertex> {
    pub indicies: Vec<u32>,
    pub verts: Vec<V>,
}

impl<V: WavefrontVertex> WavefrontMesh<V> {
    pub fn vertex_buffer(&self, ctx: &Facade) -> Result<VertexBuffer<V>, ::glium::vertex::BufferCreationError> {
        VertexBuffer::immutable(ctx, &self.verts[..])
    }

    pub fn index_buffer(&self, ctx: &Facade, prim: ::glium::index::PrimitiveType) -> Result<IndexBuffer<u32>, ::glium::index::BufferCreationError> {
        IndexBuffer::immutable(ctx, prim, &self.indicies[..])
    }
}

pub trait WavefrontVertex : Vertex {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        tex: &[[f32; 2]], 
        nor: &[[f32; 3]]) -> Self;
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct V {
    pub a_pos: [f32; 3],
}

impl WavefrontVertex for V {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        _: &[[f32; 2]], 
        _: &[[f32; 3]]) -> Self 
    {
        V {
            a_pos: pos[inds.0],
        }
    }
}

implement_vertex!(V, a_pos);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vn {
    pub a_pos: [f32; 3],
    pub a_nor: [f32; 3],
}

impl WavefrontVertex for Vn {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        _: &[[f32; 2]], 
        nor: &[[f32; 3]]) -> Self 
    {
        Vn {
            a_pos: pos[inds.0],
            a_nor: inds.2.map(|i| nor[i]).unwrap_or([0f32; 3])
        }
    }
}

implement_vertex!(Vn, a_pos, a_nor);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vt {
    pub a_pos: [f32; 3],
    pub a_tex: [f32; 2],
}

impl WavefrontVertex for Vt {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        tex: &[[f32; 2]], 
        _: &[[f32; 3]]) -> Self
    {
        Vt {
            a_pos: pos[inds.0],
            a_tex: inds.1.map(|i| tex[i]).unwrap_or([0f32; 2]),
        }
    }
}

implement_vertex!(Vt, a_pos, a_tex);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vtn {
    pub a_pos: [f32; 3],
    pub a_tex: [f32; 2],
    pub a_nor: [f32; 3],
}

impl WavefrontVertex for Vtn {
    fn load(
        inds: Inds, 
        pos: &[[f32; 3]], 
        tex: &[[f32; 2]], 
        nor: &[[f32; 3]]) -> Self 
    {
        Vtn {
            a_pos: pos[inds.0],
            a_tex: inds.1.map(|i| tex[i]).unwrap_or([0f32; 2]),
            a_nor: inds.2.map(|i| nor[i]).unwrap_or([0f32; 3]),
        }
    }
}

implement_vertex!(Vtn, a_pos, a_tex, a_nor);

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

fn add_index<V: WavefrontVertex> (
    inds: IInds, 
    verts: &mut Vec<V>,
    dedup: &mut HashMap<Inds, usize>, 
    pos: &[[f32; 3]], 
    tex: &[[f32; 2]], 
    nor: &[[f32; 3]],
    line: usize) -> Result<u32, usize>
{
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


    Ok(*dedup.entry(inds).or_insert_with(|| { 
        verts.push(V::load(inds, pos, tex, nor));
        verts.len() - 1
    }) as u32)
}

fn parse_inds(data: &str, line: usize) -> Result<IInds, usize> {
    let mut inds = data.split('/');
    let err = Err(line);

    let pos = inds.next().ok_or(line)?.parse().or(err)?;
    let tex = match inds.next() {
        Some("") => None,
        Some(v) => Some(v.parse().or(err)?),
        None => None,
    };
    let norm = match inds.next() {
        Some("") => None,
        Some(v) => Some(v.parse().or(err)?),
        None => None,
    };

    Ok((pos, tex, norm))
}

pub fn load_from_path<V: WavefrontVertex, P: AsRef<Path>>(path: P) -> Result<WavefrontMesh<V>, ()> {
    let file = File::open(path).map_err(|_| ())?;
    load(BufReader::new(file)).map_err(|_| ())
}

pub fn load<V: WavefrontVertex, R: BufRead>(read: R) -> Result<WavefrontMesh<V>, usize> {
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
                for d in line {
                    inds.push(add_index(
                        parse_inds(d, linen)?, 
                        &mut verts, 
                        &mut dedup, 
                        &pos[..], 
                        &tex[..], 
                        &nor[..],
                        linen
                    )?)
                }
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