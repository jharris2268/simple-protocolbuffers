
mod read;
mod write;

pub use read::*;
pub use write::*;


#[cfg(test)]
mod tests {
    
    use std::iter::FromIterator;
    use crate::*;
    
    #[test]
    fn test_read_all_tags() {
        let data: Vec<u8> = vec![
            8, 27, 16, 181, 254, 132, 214, 241, 2, 26, 4, 102, 114, 111, 103,
        ];
        let decoded = Vec::from_iter(IterTags::new(&data));

        let should_equal = vec![
            PbfTag::Value(1, 27),
            PbfTag::Value(2, 99233120053),
            PbfTag::Data(3, b"frog"),
        ];

        assert_eq!(decoded, should_equal);
    }

    #[test]
    fn test_read_uint32() {
        let data: Vec<u8> = vec![11, 60, 198, 127];
        let (r, p) = read_uint32(&data, 0).unwrap();
        assert_eq!(r, 188532351);
        assert_eq!(p, 4);
    }
    
    
    #[test]
    fn test_read_packed_int() {
        let data: Vec<u8> = vec![25, 155,33, 232,154,3, 0];
        let unpacked = read_packed_int(&data);
        
        assert_eq!(unpacked, vec![25, 33*128+27, 3*128*128 + 26*128+104, 0]);
    }
    
    #[test]
    fn test_extract_f64_from_u64() {
        assert_eq!(extract_f64_from_u64(4634994327930099728), 75.231);
        assert_eq!(extract_f64_from_u64(13886166140936086744), -5522.53312);
    }
    #[test]
    fn test_write_tags() {
        
        let mut res = Vec::new();
        pack_value(&mut res, 1, 27);
        pack_value(&mut res, 2, 99233120053);
        pack_data(&mut res, 3, b"frog");
        
        let should_equal: Vec<u8> = vec![
            8, 27, 16, 181, 254, 132, 214, 241, 2, 26, 4, 102, 114, 111, 103,
        ];
        
        assert_eq!(res, should_equal);
        
    }

    #[test]
    fn test_pack_uint32() {
        let mut res=Vec::new();
        write_uint32(&mut res, 188532351);
        
        assert_eq!(res, vec![11, 60, 198, 127]);
    }
    
    
    #[test]
    fn test_write_packed_int() {
        let vals = vec![25, 33*128+27, 3*128*128 + 26*128+104, 0];
        let packed = pack_int_ref(vals.iter());
        
        assert_eq!(packed, vec![25, 155,33, 232,154,3, 0]);
    }
    
}
