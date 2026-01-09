use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use std::any::{Any, TypeId};
use std::hash::{DefaultHasher, Hash, Hasher};
use tfserver::structures::s_type::{StrongType, StructureType};

#[repr(u8)]
#[derive(Serialize, Deserialize, PartialEq, Clone, Hash, Eq, TryFromPrimitive, Copy)]
pub enum ProtoLinkSType {
    RegisterRequest,
    AuthRequest,
    AuthResponse,
}

impl ProtoLinkSType {
    pub fn deserialize(val: u64) -> Box<dyn StructureType> {
        Box::new(Self::try_from(val as u8).unwrap())
    }

    pub fn serialize(refer: Box<dyn StructureType>) -> u64 {
        let res = refer.as_any().downcast_ref::<Self>().unwrap().clone() as u8 as u64;
        res
    }
}

impl StructureType for ProtoLinkSType {
    fn get_type_id(&self) -> TypeId {
        match self.clone() {
            Self::RegisterRequest => TypeId::of::<RegisterRequestStruct>(),
            Self::AuthRequest => TypeId::of::<AuthRequestStruct>(),
            Self::AuthResponse => TypeId::of::<AuthResponse>(),
        }
    }

    fn equals(&self, other: &dyn StructureType) -> bool {
        let downcast = other.as_any().downcast_ref::<Self>();
        if downcast.is_none() {
            return false;
        }
        let downcast = downcast.unwrap();
        downcast.clone() as u8 == self.clone() as u8
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::default();
        TypeId::of::<Self>().hash(&mut hasher);
        ((*self).clone() as u8 as u64).hash(&mut hasher);
        return hasher.finish();
    }

    fn clone_unique(&self) -> Box<dyn StructureType> {
        Box::new(self.clone())
    }

    fn get_deserialize_function(&self) -> Box<dyn Fn(u64) -> Box<dyn StructureType>> {
        Box::new(Self::deserialize)
    }

    fn get_serialize_function(&self) -> Box<dyn Fn(Box<dyn StructureType>) -> u64> {
        Box::new(Self::serialize)
    }
}

#[derive(Serialize, Deserialize)]
pub struct RegisterRequestStruct {
    s_type: ProtoLinkSType,
    pub name: String,
    pub login: String,
    pub password_hash_pbkdf2: String,
    pub password_salt: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthRequestStruct {
    s_type: ProtoLinkSType,
    pub login: String,
}
#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub(crate) success: bool,
    pub s_type: ProtoLinkSType,
    pub message: String,
}

impl StrongType for AuthResponse{
    fn get_s_type(&self) -> &dyn StructureType {
        &self.s_type
    }
}

impl StrongType for RegisterRequestStruct {
    fn get_s_type(&self) -> &dyn StructureType {
        &self.s_type
    }
}

impl StrongType for AuthRequestStruct {
    fn get_s_type(&self) -> &dyn StructureType {
        &self.s_type
    }
}
