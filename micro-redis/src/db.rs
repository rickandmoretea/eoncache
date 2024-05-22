use bytes::Bytes;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use tokio::time::{Duration, Instant};

/// Represents a single key-value database with optional key expiration.
#[derive(Debug)]
struct Namespace {
    entries: HashMap<String, Entry>,
    lists: HashMap<String, VecDeque<Bytes>>, 
}

/// Entry in the key-value store.
#[derive(Debug, Clone)]
struct Entry {
    data: Bytes,
}

/// A simple multi-namespace key-value store.
pub struct Db {
    namespaces: Vec<Mutex<Namespace>>,
    current_namespace_index: Mutex<usize>,  // Guarded by a Mutex for interior mutability
}


impl Db {
    pub fn new() -> Db {
        let namespaces = (0..16).map(|_| {
            Mutex::new(Namespace {
                entries: HashMap::new(),
                lists: HashMap::new(),
            })
        }).collect();
        
        Db {
            namespaces,
            current_namespace_index: Mutex::new(0), // Default to namespace 0
        }
    }


    /// Selects the namespace to operate on.
    pub fn select_namespace(&self, index: usize) -> Result<(), String> {
        let mut current_index = self.current_namespace_index.lock().unwrap();
        if index < self.namespaces.len() {
            *current_index = index;
            Ok(())
        } else {
            Err("Namespace index out of range".into())
        }
    }
    /// Retrieves the value associated with a key in the current namespace.
    pub fn get(&self, key: &str) -> Option<Bytes> {
        let ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
        ns.entries.get(key).map(|entry| entry.data.clone())
    }

    /// Sets the value for a key in the current namespace, with an optional expiration time.
    pub fn set(&self, key: String, value: Bytes) {
        let mut ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
        ns.entries.insert(key, Entry { data: value });
    }
    /// Sets the value for a key in the current namespace, with an optional expiration time.
    pub fn exists(&self, key: &str) -> bool {
        let ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
        ns.entries.contains_key(key)
    }

    pub fn lpush(&self, key: String, value: Bytes) -> Result<usize, &'static str>{
        let mut ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
        match ns.entries.get(&key) {
            Some(_) => Err("key holds a different type"),
            None => {
                let list = ns.lists.entry(key).or_insert_with(VecDeque::new);
                list.push_front(value);
                Ok(list.len())
            },
        }
    }

    pub fn rpush(&self, key: String, value: Bytes) -> Result<usize, &'static str> {
        let mut ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
        match ns.entries.get(&key) {
            Some(_) => Err("key holds a different type"),
            None => {
                let list = ns.lists.entry(key).or_insert_with(VecDeque::new);
                list.push_back(value);
                Ok(list.len())
            },
        }
    }

    pub async fn blpop(&self, keys: Vec<String>, timeout: Duration) -> Option<(String, Bytes)> {
        let start = Instant::now();
        while Instant::now().duration_since(start) < timeout {
            for key in &keys {
                let mut ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
                if let Some(queue) = ns.lists.get_mut(key) {
                    if let Some(value) = queue.pop_front() {
                        return Some((key.clone(), value));
                    }
                }
            }
            // Sleep for a small interval before checking again.
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        None
    }

    pub async fn brpop(&self, keys: Vec<String>, timeout: Duration) -> Option<(String, Bytes)> {
        let start = Instant::now();
        while Instant::now().duration_since(start) < timeout {
            for key in &keys {
                let mut ns = self.namespaces[*self.current_namespace_index.lock().unwrap()].lock().unwrap();
                if let Some(queue) = ns.lists.get_mut(key) {
                    if let Some(value) = queue.pop_back() {
                        return Some((key.clone(), value));
                    }
                }
            }
            // Sleep for a small interval before checking again.
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        None
    }


}
