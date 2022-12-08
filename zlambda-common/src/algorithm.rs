pub fn next_key<'a, T>(iterator: T) -> u64
where
    T: Iterator<Item = &'a u64>,
{
    let mut keys = iterator.collect::<Vec<_>>();
    keys.sort_unstable();

    let mut id = 0;

    for key in keys {
        if id != *key {
            break;
        }

        id += 1;
    }

    id
}
