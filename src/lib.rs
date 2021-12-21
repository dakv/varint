const MSB: u8 = 0b1000_0000;
const MSB_U64: u64 = 0b1000_0000;
const DROP_MSB: u8 = 0b0111_1111;

#[inline]
fn required_encoded_space_unsigned(mut v: u64) -> usize {
    if v == 0 {
        return 1;
    }
    let mut c = 0;
    while v > 0 {
        c += 1;
        v >>= 7;
    }
    c
}

/// Varint trait
pub trait EncodeVar: Sized + Copy {
    fn required_size(self) -> usize;
    fn real_encode(self, dst: &mut [u8]);
    fn encode_varint(self) -> Vec<u8> {
        let mut v = vec![0; self.required_size()];
        self.real_encode(&mut v);
        v
    }
    fn decode_varint(src: &[u8]) -> (Self, usize);
}

impl EncodeVar for u64 {
    fn required_size(self) -> usize {
        required_encoded_space_unsigned(self)
    }
    fn real_encode(self, v: &mut [u8]) {
        let mut i = 0;
        let mut t = self;
        while t >= MSB_U64 {
            v[i] = t as u8 & (MSB - 1) | MSB;
            i += 1;
            t >>= 7;
        }
        v[i] = t as u8;
    }
    fn decode_varint(src: &[u8]) -> (Self, usize) {
        let mut result: u64 = 0;
        let mut shift = 0;

        for b in src.iter() {
            let msb_dropped = b & DROP_MSB;
            result |= (msb_dropped as u64) << shift;
            shift += 7;
            if b & MSB == 0 || shift > (10 * 7) {
                break;
            }
        }
        (result, (shift / 7) as usize)
    }
}
/// todo signed
macro_rules! impl_varint {
    ($t:ty, unsigned) => {
        impl EncodeVar for $t {
            fn required_size(self) -> usize {
                required_encoded_space_unsigned(self as u64)
            }
            fn real_encode(self, v: &mut [u8]) {
                (self as u64).real_encode(v);
            }
            fn encode_varint(self) -> Vec<u8> {
                (self as u64).encode_varint()
            }
            fn decode_varint(src: &[u8]) -> (Self, usize) {
                let (n, s) = u64::decode_varint(src);
                (n as Self, s)
            }
        }
    };
}

impl_varint!(usize, unsigned);
impl_varint!(u32, unsigned);
impl_varint!(u16, unsigned);
impl_varint!(u8, unsigned);

#[cfg(test)]
mod test {
    use super::EncodeVar;

    #[test]
    fn test_var_u32() {
        let mut value = 1u32;
        while value < std::u32::MAX / 10 {
            value *= 10;
            let v = value.encode_varint();
            let expected_len = v.len();
            let (result, l) = u32::decode_varint(&v);
            assert_eq!(l, expected_len);
            assert_eq!(result, value);
        }
    }

    #[test]
    fn test_var_u64() {
        let mut value = 1u64;
        while value < std::u64::MAX / 10 {
            value *= 10;
            let v = value.encode_varint();
            let expected_len = v.len();
            let (result, l) = u64::decode_varint(&v);
            assert_eq!(l, expected_len);
            assert_eq!(result, value);
        }
    }
}
