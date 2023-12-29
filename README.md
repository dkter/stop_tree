This is a quick tool I made to download stops from multiple GTFS feeds, insert them into a k-d tree by latitude and longitude with [kiddo](https://github.com/sdd/kiddo) and save it to a file using [rkyv](https://github.com/rkyv/rkyv). The resulting file can be used as follows -

```rust
use std::io::Read;
use std::fs::File;
use std::path::Path;
use kiddo::{KdTree, SquaredEuclidean};

#[derive(Debug, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
struct Stop {
    agency: String,
    stop_id: String,
    stop_code: Option<String>,
    stop_name: String,
    tts_stop_name: Option<String>,
    stop_desc: Option<String>,
    stop_lat: f64,
    stop_lon: f64,
    zone_id: Option<String>,
    stop_url: Option<String>,
    location_type: Option<String>,
    parent_station: Option<String>,
    stop_timezone: Option<String>,
    wheelchair_boarding: Option<u32>,
    level_id: Option<String>,
    platform_code: Option<String>,
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let fpath = &args[1];
    let path = Path::new(&fpath);

    let mut file = File::open(&path).unwrap();
    let mut bytes = vec![];
    file.read_to_end(&mut bytes).unwrap();
    let deserialized = unsafe {
        rkyv::from_bytes_unchecked::<(Vec<Stop>, KdTree<f64, 2>)>(&bytes)
            .expect("Failed to deserialize KdTree")
    };
    let (stops, tree) = deserialized;

    let nearest_n: Vec<_> = tree.nearest_n::<SquaredEuclidean>(&[43.6528525f64,-79.3982886f64], 5);
    for neighbour in nearest_n {
        let stop = &stops[neighbour.item as usize];
        println!("{} away  -  {:?}", neighbour.distance, stop);
    }
}
```

I'm using this to get stops nearby to a specific location in [Departure Board](https://github.com/dkter/departure-board), since all the existing APIs I could find were too slow or unreliable. Keep in mind that this uses the assumption that latitudes/longitudes can be treated as rectangular coordinates on a small scale, which works fine on the scale of bus stops -- however, if your transit system straddles the International Date Line or something things might get weird.
