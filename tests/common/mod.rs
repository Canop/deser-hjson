#![allow(unused_macros)]

// allows writing vo!["a", "b"] to build a vec of strings
macro_rules! vo {
    ($($item:literal),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut vec = Vec::new();
        $(
            vec.push($item.to_owned());
        )*
        vec
    }}
}

// allows writing mo!{"a":"b", "c":"d"} to build a map of strings to strings
macro_rules! mo {
    ($($key:literal:$value:literal),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut map = HashMap::new();
        $(
            map.insert($key.to_owned(), $value.to_owned());
        )*
        map
    }}
}

