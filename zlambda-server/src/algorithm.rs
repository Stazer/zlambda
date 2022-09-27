use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn next_key<T>(collection: &HashMap<u64, T>) -> u64 {
    let mut keys = collection.keys().map(|x| *x as u64).collect::<Vec<_>>();
    keys.sort_unstable();
    let mut id = 0;

    for key in keys {
        if id != key {
            break;
        }

        id = id + 1;
    }

    id
}
