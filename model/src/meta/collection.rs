
/* **************************************************************************************************************** */
/* **************************************************** MODS ****************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** IMPORTS **************************************************** */
/* **************************************************************************************************************** */
use serde::{Serialize, de::Visitor, Deserialize, Deserializer, Serializer, ser::SerializeMap};
use std::{fmt::{self, Debug}, marker::PhantomData, ops::Index};

use crate::UserMetaId;

use super::{MetaType, UserMetaAccess};

/* **************************************************************************************************************** */
/* ******************************************** STATICS/CONSTS/TYPES ********************************************** */
/* **************************************************************************************************************** */
pub type UserMetaCollection = MetaCollection<UserMetaAccess, UserMetaId>;

/* **************************************************************************************************************** */
/* **************************************************** MACROS **************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* *************************************************** STRUCTS **************************************************** */
/* **************************************************************************************************************** */
#[derive(Clone, Debug, PartialEq)]
pub struct MetaInformation<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString>  {
    pub id: Option<I>,
    pub name: String,
    pub meta: MetaType<T>
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct MetaCollection<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString> {
    items: Vec<MetaInformation<T, I>>
}

pub struct MetaCollectionIterator<'a, T, I>
    where
        T: Default + Debug + PartialEq + Clone + From<i32>,
        I: Default + ToString + PartialEq {
    iter: core::slice::Iter<'a, MetaInformation<T, I>>
}

#[derive(Default)]
struct MetaCollectionVisitor<T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>, I: Default + ToString> {
    _marker1: PhantomData<T>,
    _marker2: PhantomData<I>
}

/* **************************************************************************************************************** */
/* **************************************************** ENUMS ***************************************************** */
/* ************************************************** FUNCTIONS *************************************************** */
/* *************************************************** TRAITS ***************************************************** */
/* **************************************************************************************************************** */

/* **************************************************************************************************************** */
/* ************************************************* IMPLEMENTS *************************************************** */
/* **************************************************************************************************************** */
impl<T, I> MetaCollection<T, I> where
    T: Default + Debug + PartialEq + Clone + From<i32>,
    I: Default + ToString + PartialEq {
    
    pub fn new() -> Self {
        Self {
            items: Vec::new()
        }
    }

    pub fn add(&mut self, key: String, value: MetaType<T>) {
        self.items.push(MetaInformation { id: None, name: key, meta: value })
    }

    pub fn add_with_id(&mut self, id: I,  key: String, value: MetaType<T>) {
        self.items.push(MetaInformation { id: Some(id), name: key, meta: value })
    }

    pub fn add_item(&mut self, item: MetaInformation<T, I>) {
        self.items.push(item)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn remove_with_id(&mut self, id: &I) {
        let _tmp_id = Some(id);
        let index = self.items.iter().position(|x| x.id.as_ref() == _tmp_id);

        // Meta found
        if let Some(index) = index {
            self.items.remove(index);
        }
    }

    pub fn remove_with_name(&mut self, name: &str) {
        let index = self.items.iter().position(|x| &x.name == name);

        // Meta found
        if let Some(index) = index {
            self.items.remove(index);
        }
    }

    pub fn get_with_id(&self, id: &I) -> Option<&MetaInformation<T, I>> {
        let _tmp_id = Some(id);
        self.items.iter().find(|item| item.id.as_ref() == _tmp_id)
    }

    pub fn get_with_name(&self, name: &str) -> Option<&MetaInformation<T, I>> {
        self.items.iter().find(|item| &item.name == name)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn iter(&self) -> MetaCollectionIterator<T, I> {
        MetaCollectionIterator {
            iter: self.items.iter()
        } 
    }
}

/* **************************************************************************************************************** */
/* ********************************************** TRAIT IMPLEMENTS ************************************************ */
/* **************************************************************************************************************** */
impl<'de, T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>, I: ToString + Default + PartialEq> Deserialize<'de> for MetaCollection<T, I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(MetaCollectionVisitor::default())
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString + PartialEq> Serialize for MetaCollection<T, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.items.len()))?;

        for meta_information in self.items.iter() {
            map.serialize_entry(&meta_information.name, &meta_information.meta)?;
        }
        
        map.end()
    }
}

impl<'de, T: Default + Debug + PartialEq + Clone + From<i32> + Into<i32>, I: Default + ToString + PartialEq> Visitor<'de> for MetaCollectionVisitor<T, I> {
    type Value = MetaCollection<T, I>;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("meta collection")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>, {
        
        let mut collection = MetaCollection::new();
        while let Some(key) = map.next_key::<String>()? {
            collection.add(key, map.next_value()?);
        }

        Ok(collection)
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString + PartialEq> IntoIterator for MetaCollection<T, I> {
    type Item = MetaInformation<T, I>;

    type IntoIter = ::std::vec::IntoIter<MetaInformation<T, I>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString + PartialEq> Iterator for MetaCollectionIterator<'a, T, I> {

    type Item = &'a MetaInformation<T, I>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString + PartialEq> FromIterator<MetaInformation<T, I>> for MetaCollection<T, I> {
    fn from_iter<A: IntoIterator<Item = MetaInformation<T, I>>>(iter: A) -> Self {
        let mut collection = MetaCollection::<T, I>::new();

        for item in iter {
            collection.add_item(item);
        }

        collection
    }
}

impl<T: Default + Debug + PartialEq + Clone + From<i32>, I: Default + ToString + PartialEq> Index<usize> for MetaCollection<T, I>  {
    type Output = MetaInformation<T, I>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

/* **************************************************************************************************************** */
/* ************************************************* MACROS CALL ************************************************** */
/* ************************************************** UNIT TESTS ************************************************** */
/* **************************************************************************************************************** */
