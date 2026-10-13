use glam::IVec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HitInfo {
    pub(crate) pos: IVec3,
    pub(crate) face: HitFace,
    pub(crate) normal: IVec3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HitFace {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl HitFace {
    fn normal(&self) -> IVec3 {
        match self {
            HitFace::PosX => IVec3::new(1, 0, 0),
            HitFace::NegX => IVec3::new(-1, 0, 0),
            HitFace::PosY => IVec3::new(0, 1, 0),
            HitFace::NegY => IVec3::new(0, -1, 0),
            HitFace::PosZ => IVec3::new(0, 0, 1),
            HitFace::NegZ => IVec3::new(0, 0, -1),
        }
    }
}

impl HitInfo {
    pub(crate) fn new(pos: IVec3, face: HitFace) -> Self {
        Self {
            pos,
            face,
            normal: face.normal(),
        }
    }
}
