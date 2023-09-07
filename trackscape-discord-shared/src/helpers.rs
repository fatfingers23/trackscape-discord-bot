use xxhash_rust::xxh3::Xxh3;

pub fn hash_string(code: String) -> String {
    let mut hasher = Xxh3::new();
    hasher.update(code.as_ref());
    hasher.digest().to_string()
}