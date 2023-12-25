use std::io::Write;
use std::error::Error;
use std::fs::File;
use futures::future;
use kiddo::{KdTree, SquaredEuclidean};
use phf::phf_map;

static FEEDS: phf::Map<&'static str, &'static str> = phf_map! {
    "ttc" => "http://opendata.toronto.ca/toronto.transit.commission/ttc-routes-and-schedules/OpenData_TTC_Schedules.zip",
    "upexpress" => "https://assets.metrolinx.com/raw/upload/v1682367798/Documents/Metrolinx/Open%20Data/UP-GTFS.zip",
    "gotransit" => "https://assets.metrolinx.com/raw/upload/v1683228856/Documents/Metrolinx/Open%20Data/GO-GTFS.zip",
    "viarail" => "https://www.viarail.ca/sites/all/files/gtfs/viarail.zip",
    "yrt" => "https://www.yrt.ca/google/google_transit.zip",
    "miway" => "https://www.miapp.ca/GTFS/google_transit.zip?#",
    "brampton" => "http://www.brampton.ca/EN/City-Hall/OpenGov/Open-Data-Catalogue/Documents/Google_Transit.zip",
    "durham" => "https://maps.durham.ca/OpenDataGTFS/GTFS_Durham_TXT.zip",
    "grt" => "https://www.regionofwaterloo.ca/opendatadownloads/GRT_GTFS.zip",
    "hsr" => "http://googlehsrdocs.hamilton.ca/",
    "guelph" => "http://guelph.ca/uploads/google/google_transit.zip",
    "burlington" => "https://opendata.burlington.ca/gtfs-rt/GTFS_Data.zip",
    "oakville" => "https://www.arcgis.com/sharing/rest/content/items/d78a1c1ad6a940009de8b68839a8f606/data",
    "barrie" => "http://www.myridebarrie.ca/gtfs/Google_transit.zip",
    "niagara" => "https://maps.niagararegion.ca/googletransit/NiagaraRegionTransit.zip",
    "milton" => "http://metrolinx.tmix.se/gtfs/gtfs-milton.zip",
};

#[derive(Debug, serde::Serialize, serde::Deserialize, rkyv::Archive, rkyv::Deserialize, rkyv::Serialize, PartialEq)]
struct Stop {
    stop_id: String,
    stop_code: Option<String>,
    stop_name: String,
    tts_stop_name: Option<String>,
    stop_desc: Option<String>,
    stop_lat: f32,
    stop_lon: f32,
    zone_id: Option<String>,
    stop_url: Option<String>,
    location_type: Option<String>,
    parent_station: Option<String>,
    stop_timezone: Option<String>,
    wheelchair_boarding: Option<u32>,
    level_id: Option<String>,
    platform_code: Option<String>,
}

async fn download_feed(
    client: &reqwest::Client, agency: &str, url: &str
) -> Result<Vec<Stop>, Box<dyn Error>> {
    println!("Downloading {}", agency);
    let resp = client.get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/50.0.2661.102 Safari/537.36")
        .send()
        .await?;
    // let out_filename = tmp_dir.path().join(format!("{}.txt", agency));
    // let mut out_file = File::create(out_filename)?;
    let content = resp.bytes().await?;

    println!("Zipping {}", agency);

    let reader = std::io::Cursor::new(content);
    let mut zip = zip::ZipArchive::new(reader).unwrap();

    let file = zip.by_name("stops.txt")?;
    let mut reader = csv::Reader::from_reader(file);

    let mut stops = Vec::new();
    for result in reader.deserialize() {
        let stop: Stop = result?;
        stops.push(stop);
    }
    println!("Done with {}", agency);
    Ok(stops)
}

async fn download_feeds() -> Result<Vec<Stop>, Box<dyn Error>> {
    let mut stops = Vec::new();
    // let tmp_dir = tempfile::Builder::new().prefix("stop_downloader").tempdir()?;
    let client = reqwest::Client::new();

    // for (agency, &url) in FEEDS.iter() {
        
    // }
    let stop_maps = future::join_all(
        FEEDS.into_iter().map(|(agency, url)| {
            let client = &client;
            async move {
                download_feed(client, &agency, &url).await
            }
        })
    ).await;

    for stop_map in stop_maps {
        stops.extend(stop_map?);
    }

    Ok(stops)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let stops = download_feeds().await?;

    // todo: kdtree

    let bytes = rkyv::to_bytes::<_, 256>(&stops).unwrap();
    let mut file = File::create("stops.bin")?;
    Ok(file.write_all(&bytes)?)
}
