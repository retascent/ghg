use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io};

use image::EncodableLayout;
use reqwest::Url;
use {regex, reqwest, scraper};

fn main() -> Result<(), Box<dyn Error>> {
	let args: Vec<String> = env::args().collect();
	assert_eq!(args.len(), 5);

	let url: Url =
		Url::parse(args[1].as_str()).expect(format!("Invalid url: {:?}", args[1]).as_str());
	let link_container_class = &args[2];
	let regex_arg = &args[3];
	let out_location = Path::new(&args[4]);

	let a_regex = regex::Regex::new(regex_arg.as_str())
		.expect(format!("Invalid regex specification: {}", regex_arg).as_str());

	let page = reqwest::blocking::get(url.as_str())
		.expect(format!("Failed to get URL {}", url).as_str())
		.text()
		.expect(format!("Failed to extract text from URL {}", url).as_str());
	let parsed = scraper::Html::parse_document(page.as_str());

	let container_class_str = format!(".{link_container_class}");
	let container_selector = scraper::Selector::parse(container_class_str.as_str())
		.expect("Failed to create class selector");

	let a_selector = scraper::Selector::parse("a")?;

	let mut links: Vec<String> = Vec::new();
	let mut num_skipped: usize = 0;
	for link_container in parsed.select(&container_selector) {
		for hyperlink in link_container.select(&a_selector) {
			if let Some(href) = hyperlink.value().attr("href") {
				if a_regex.is_match(href) {
					links.push(href.to_owned());
				} else {
					num_skipped += 1;
				}
			}
		}
	}

	println!("Found {} links (skipped {})", links.len(), num_skipped);
	println!("Ensuring {:?} exists", out_location);

	fs::create_dir_all(out_location)
		.expect(format!("Failed to create out location {:?}", out_location).as_str());

	let user_agent = "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:47.0) Gecko/20100101 Firefox/47.0";

	let client = reqwest::blocking::ClientBuilder::new()
		.user_agent(user_agent)
		.build()?;

	for (index, link) in links.iter().enumerate() {
		if link.starts_with("/") {
			let new_url = url.join(&link).expect("Failed to join link");
			println!("Downloading {} of {}: {}", index + 1, links.len(), new_url);
			let file_name =
				new_url.path_segments().expect("Invalid path").next_back().expect("Invalid path");
			let file_path = out_location.join(file_name);

			if file_path.exists() {
				println!("File exists! Skipping... {:?}", &file_path);
				continue;
			}

			// let mut file = fs::OpenOptions::new()
			// 	.create(true)
			// 	.write(true)
			// 	.open(&file_path)
			// 	.expect(format!("Failed to open file for writing: {:?}",
			// &file_path).as_str());

			let mut file = File::create(&file_path)
				.expect(format!("Failed to open file for writing: {:?}", &file_path).as_str());

			let contents = client.get(new_url.as_str()).send()
				.expect(format!("Failed to get URL {}", new_url).as_str())
				.text()
				.unwrap();

			println!("RESULT: {}", contents);

			// io::copy(&mut contents.as_bytes(), &mut file)
			// 	.expect(format!("Failed to write file {:?}", &file_path).as_str());
		} else {
			println!("Error: Unsure how to download link: {}", link);
		}
	}

	Ok(())
}
