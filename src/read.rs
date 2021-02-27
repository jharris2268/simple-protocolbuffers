
use std::io;
use std::io::ErrorKind;

#[derive(PartialEq, Debug)]
pub enum PbfTag<'a> {
    Value(u64, u64),
    Data(u64, &'a [u8]),
    Null,
}

pub fn read_uint32(data: &[u8], pos: usize) -> io::Result<(u64, usize)> {
    if (pos + 4) > data.len() {
        return Err(io::Error::new(ErrorKind::Other, "too short"));
    }
    let mut res: u64 = 0;
    //assert!(pos+3 < data.len());

    res |= data[pos + 3] as u64;
    res |= (data[pos + 2] as u64) << 8;
    res |= (data[pos + 1] as u64) << 16;
    res |= (data[pos + 0] as u64) << 24;

    Ok((res, pos + 4))
}
pub fn read_uint64(data: &[u8], pos: usize) -> io::Result<(u64, usize)> {
    if (pos + 8) > data.len() {
        return Err(io::Error::new(ErrorKind::Other, "too short"));
    }
    let mut res: u64 = 0;
    //assert!(pos+3 < data.len());

    res |= data[pos + 7] as u64;
    res |= (data[pos + 6] as u64) << 8;
    res |= (data[pos + 5] as u64) << 16;
    res |= (data[pos + 4] as u64) << 24;
    res |= (data[pos + 3] as u64) << 32;
    res |= (data[pos + 2] as u64) << 40;
    res |= (data[pos + 1] as u64) << 48;
    res |= (data[pos + 0] as u64) << 56;
    Ok((res, pos + 8))
}

pub fn un_zig_zag(uv: u64) -> i64 {
    let x = (uv >> 1) as i64;
    if (uv & 1) != 0 {
        return x ^ -1;
    }
    x
}

pub fn read_varint(data: &[u8], pos: usize) -> (u64, usize) {
    let mut res: u64 = 0;
    let mut i = 0;
    loop {
        if i >= 10 {
            break;
        }
        let x = data[pos + i];
        let y = (x & 127) as u64;
        res |= y << (7 * i);

        if (x & 128) == 0 {
            return (res, pos + i + 1);
        }
        i += 1;
    }
    (res, pos + 10)
}

pub fn read_data<'a>(data: &'a [u8], pos: usize) -> (&'a [u8], usize) {
    let (ln, pos) = read_varint(data, pos);

    let l = ln as usize;
    (&data[pos..pos + l], pos + l)
}

pub fn read_tag<'a>(data: &'a [u8], pos: usize) -> (PbfTag<'a>, usize) {
    let (t, pos) = read_varint(data, pos);

    if t == 0 {
        return (PbfTag::Null, pos);
    }
    if (t & 7) == 0 {
        let (v, pos) = read_varint(data, pos);
        return (PbfTag::Value(t >> 3, v), pos);
    } else if (t & 7) == 1 {
        let (v, pos) = read_uint64(data, pos).expect("!!");
        return (PbfTag::Value(t >> 3, v), pos);
    } else if (t & 7) == 2 {
        let (s, pos) = read_data(data, pos);
        return (PbfTag::Data(t >> 3, s), pos);
    } else if (t & 7) == 5 {
        let (v, pos) = read_uint32(data, pos).expect("!!");
        return (PbfTag::Value(t >> 3, v), pos);
    } 
    (PbfTag::Null, pos)
}

pub fn extract_f64_from_u64(v: u64) -> f64 {
    unsafe { *(&v as *const u64 as *const f64) }
}

pub struct IterTags<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> IterTags<'a> {
    pub fn new(data: &'a [u8]) -> IterTags<'a> {
        IterTags { data: data, pos: 0 }
    }
}

impl<'a> Iterator for IterTags<'a> {
    type Item = PbfTag<'a>;

    fn next(&mut self) -> Option<PbfTag<'a>> {
        if self.pos < self.data.len() {
            let (t, npos) = read_tag(self.data, self.pos);
            self.pos = npos;
            return Some(t);
        }
        None
    }
}


fn count_packed_len(data: &[u8]) -> usize {
    let mut pos = 0;
    let mut count = 0;

    while pos < data.len() {
        pos = read_varint(data, pos).1;
        count += 1;
    }
    count
}

pub struct DeltaPackedInt<'a> {
    data: &'a [u8],
    curr: i64,
    pos: usize,
}

impl DeltaPackedInt<'_> {
    pub fn new(data: &'_ [u8]) -> DeltaPackedInt<'_> {
        DeltaPackedInt {
            data,
            curr: 0,
            pos: 0,
        }
    }
}

impl Iterator for DeltaPackedInt<'_> {
    type Item = i64;

    fn next(&mut self) -> Option<i64> {
        if self.pos < self.data.len() {
            let (t, npos) = read_varint(&self.data, self.pos);
            let p = un_zig_zag(t);
            self.curr += p;

            self.pos = npos;

            Some(self.curr)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = count_packed_len(&self.data);
        (s, Some(s))
    }
}
impl ExactSizeIterator for DeltaPackedInt<'_> {
    fn len(&self) -> usize {
        count_packed_len(&self.data)
    }
}

pub struct PackedInt<'a> {
    data: &'a [u8],
    pos: usize,
}
impl PackedInt<'_> {
    pub fn new(data: &[u8]) -> PackedInt<'_> {
        PackedInt { data, pos: 0 }
    }
}

impl Iterator for PackedInt<'_> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.pos < self.data.len() {
            let (t, npos) = read_varint(&self.data, self.pos);
            self.pos = npos;

            Some(t)
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = count_packed_len(&self.data);
        (s, Some(s))
    }
}

impl ExactSizeIterator for PackedInt<'_> {
    fn len(&self) -> usize {
        count_packed_len(&self.data)
    }
}

pub fn read_delta_packed_int(data: &[u8]) -> Vec<i64> {    
    DeltaPackedInt::new(data).collect()

}

pub fn read_packed_int(data: &[u8]) -> Vec<u64> {
    PackedInt::new(data).collect()
}

