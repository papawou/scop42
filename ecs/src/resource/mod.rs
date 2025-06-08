use std::{any::TypeId, collections::HashMap};

pub mod error;
pub mod traits;

use error::Error;
use traits::Resource;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ResourceStorage(HashMap<TypeId, Box<dyn Resource>>);

impl ResourceStorage {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn add<T: Resource>(&mut self, resource: T) {
        let type_id = TypeId::of::<T>();
        self.0.entry(type_id).or_insert(Box::new(resource));
    }

    pub fn get<T: Resource>(&self) -> Result<Option<&T>> {
        let type_id = TypeId::of::<T>();

        match self.0.get(&type_id) {
            Some(res) => match res.as_ref().as_any().downcast_ref() {
                Some(cast) => Ok(Some(cast)),
                None => Err(Box::new(Error::TypeMismatch)),
            },
            None => Ok(None),
        }
    }

    pub fn get_mut<T: Resource>(&mut self) -> Result<Option<&mut T>> {
        let type_id = TypeId::of::<T>();
        match self.0.get_mut(&type_id) {
            Some(res) => match res.as_mut().as_any_mut().downcast_mut() {
                Some(cast) => Ok(Some(cast)),
                None => Err(Box::new(Error::TypeMismatch)),
            },
            None => Ok(None),
        }
    }

    pub fn remove<T: Resource>(&mut self) -> Result<Option<T>> {
        let type_id = TypeId::of::<T>();
        match self.0.remove(&type_id) {
            Some(res) => match res.into_any().downcast::<T>() {
                Ok(res) => Ok(Some(*res)),
                Err(err) => Err(Box::new(Error::Custom(format!("{:?}", err)))),
            },
            None => Ok(None),
        }
    }
}
