extern crate dirs;
extern crate quick_xml;
extern crate reqwest;

use quick_xml::events;
use std::error;
use std::fs;
use std::io;
use std::path;
use std::process;

const BING: &str = "https://www.bing.com";
const PYWAL: &str = "/home/asad/.local/bin/wal";
const WALLPAPAERS: &str = "BingWallpaper";

struct BingImage {
    url: String,
    name: String,
    startdate: String,
}

fn market() -> String {
    "en-US".to_string()
}

fn bing_xml_url() -> String {
    format!(
        "{}/HPImageArchive.aspx?format=xml&idx=0&n=1&mkt={}",
        BING,
        market()
    )
}

fn bing_xml() -> Result<String, Box<dyn error::Error>> {
    Ok(reqwest::get(&bing_xml_url())?.text()?)
}

fn bing_image_info() -> BingImage {
    let xml = bing_xml().unwrap();
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.trim_text(true);

    let mut txt_url = Vec::new();
    let mut txt_startdate = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(events::Event::Start(ref e)) if e.name() == b"url" => {
                txt_url.push(reader.read_text(b"url", &mut Vec::new()));
            }
            Ok(events::Event::Start(ref e)) if e.name() == b"startdate" => {
                txt_startdate.push(reader.read_text(b"startdate", &mut Vec::new()));
            }
            Ok(events::Event::Eof) => break,
            _ => (),
        }
    }

    let partial_url = txt_url[0].as_ref().unwrap();
    let url = format!("{}{}", BING, partial_url);
    let name = partial_url[11..(partial_url.find(".jpg").unwrap() + 4)].to_string();
    let startdate = txt_startdate[0].as_ref().unwrap().to_string();
    BingImage {
        url,
        name,
        startdate,
    }
}

fn save_bing_image() -> Result<String, Box<dyn error::Error>> {
    let image = bing_image_info();
    let filename;
    let mut dest = {
        let fname = format!("{}-{}", image.startdate, image.name);

        println!("file to download: {}", fname);
        let dir = dirs::picture_dir().unwrap();
        let pictures = dir.to_str().unwrap();
        let fname = path::Path::new(pictures).join(WALLPAPAERS).join(fname);
        filename = String::from(fname.to_str().unwrap());
        println!("will be located under: {:?}", fname);
        fs::File::create(fname)?
    };

    let mut response = reqwest::get(&image.url)?;
    io::copy(&mut response, &mut dest)?;

    Ok(filename)
}

fn run() {
    match save_bing_image() {
        Ok(path_string) => {
            println!("Successful: {}", path_string);
            let bing_image_path = String::from(path_string);
            process::Command::new(PYWAL)
                .arg("-i")
                .arg(bing_image_path)
                .output()
                .expect("Failed to generate and apply themes.");
        }
        Err(e) => println!("Error occured: {}", e),
    }
}

fn main() {
    run();
}
