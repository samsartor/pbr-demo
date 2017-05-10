use std::io::Read;

use define::CtrPoint;

pub fn read_ctrpts<R: Read>(input: &mut R) -> (Vec<CtrPoint>, Vec<u32>) {
    let mut buf = String::new();
    input.read_to_string(&mut buf).unwrap();
    let mut lines = buf.lines();

    let patch_count = lines.next().unwrap().parse().unwrap();
    let mut inds = Vec::with_capacity(patch_count);

    for _ in 0..patch_count {
        inds.extend(lines
                        .next()
                        .unwrap()
                        .split(',')
                        .map(|v| { let v: u32 = v.parse().unwrap(); v - 1 }));

    }

    assert_eq!(inds.len(), patch_count * 16);

    let vert_count = lines.next().unwrap().parse().unwrap();
    let mut verts = Vec::with_capacity(vert_count);

    for _ in 0..vert_count {
        let mut vals = lines
            .next()
            .unwrap()
            .split(',')
            .map(|v| v.parse().unwrap());
        verts.push(CtrPoint {
                       pos: [vals.next().unwrap(),
                             vals.next().unwrap(),
                             vals.next().unwrap()],
                   });

    }

    (verts, inds)
}
