use std::sync::{ Arc, Mutex };

use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
pub(crate) struct Size {
    pub x: u32,
    pub y: u32,
}

impl Size {
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            x: x,
            y: y,
        }
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        from_json(text)
    }
}

pub fn from_json<T>(text: &str) -> Result<T, String> where T: serde::de::DeserializeOwned {
    let result: Result<T, serde_json::Error> = serde_json::from_str(text);
    if result.is_ok() {
        return Ok(result.unwrap());
    }
    let err_string = result.err().unwrap().to_string();
    Err(err_string)
}

pub fn get_object<T, P>(arr: &Arc<Mutex<Vec<Arc<Mutex<T>>>>>, predicate: P) -> Option<Arc<Mutex<T>>>
    where P: FnMut(&Arc<Mutex<T>>) -> bool
{
    let guard = arr.lock().unwrap();
    let index = find_index(&guard, predicate);
    if index.is_some() {
        let result = Some(guard[index.unwrap()].clone());
        drop(guard);
        return result;
    }
    None
}
pub fn find_index<T, P>(arr: &Vec<Arc<Mutex<T>>>, predicate: P) -> Option<usize>
    where P: FnMut(&Arc<Mutex<T>>) -> bool
{
    arr.iter().position(predicate)
}

pub fn get_unique_id(id_counter: &Arc<Mutex<i32>>) -> i32 {
    let mut guard = id_counter.lock().unwrap();
    let id = *guard;
    *guard += 1;
    id
}
