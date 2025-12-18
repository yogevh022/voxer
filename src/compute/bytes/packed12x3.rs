use glam::UVec3;


#[derive(Debug, Clone, Copy)]
pub struct Packed12x3 {
    pub xyz: u32,
    pub z_hi: u32,
}

impl Packed12x3 {
    pub fn pack(xyz: UVec3) -> Self {
        Self {
            xyz: xyz.x | xyz.y << 12 | xyz.z << 24,
            z_hi: xyz.z >> 8,
        }
    }

    pub fn unpack(&self) -> UVec3 {
        UVec3 {
            x: self.xyz & 0xFFF,
            y: (self.xyz >> 12) & 0xFFF,
            z: (self.xyz >> 24) | self.z_hi << 8,
        }
    }

    pub fn unpack_from(packed: impl Into<[u32 ;2]>) -> UVec3 {
        let packed = packed.into();
        UVec3 {
            x: packed[0] & 0xFFF,
            y: (packed[0] >> 12) & 0xFFF,
            z: (packed[0] >> 24) | packed[1] << 8,
        }
    }
}

impl Into<[u32; 2]> for Packed12x3 {
    fn into(self) -> [u32; 2] {
        [self.xyz, self.z_hi]
    }
}

impl From<[u32; 2]> for Packed12x3 {
    fn from(value: [u32; 2]) -> Self {
        Self { xyz: value[0], z_hi: value[1] }
    }
}