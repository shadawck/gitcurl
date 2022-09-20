//// WORKING URL
// cgit https://github.com/shadawck/rust-trend
// cgit https://github.com/shadawck/rust-trend.git
// cgit github.com/shadawck/rust-trend
// cgit github.com/shadawck/rust-trendgit
// cgit shadawck:rust-trend

//// OPTIONS
// -z : ZIP : extract_zip the zip source code but do not extract_zip the zip
// -o : if -z just rename the zip file; if -extract_zip (default) extract_zip in a specific folder
// -b : branch name to fetch

// cgit -z shadawck:rust-trend
// cgit -z shadawck:rust-trend -b dev

// cgit -z shadawck:rust-trend -o new_zip_name.zip
// cgit -z shadawck:rust-trend -o new_zip_name

// cgit -z shadawck:rust-trend -o /tmp/new_zip_name.zip
// cgit -z shadawck:rust-trend -o ./new_folder/new_zip_name.zip

use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::panic;

use clap::{crate_version, value_parser, App, Arg};
use curl::easy::{Easy2, Handler, WriteError};
use fs_extra::dir::{move_dir, CopyOptions};

// Helper Type
type UserName = str;
type RepoName = str;

struct CollectorMemFile(Vec<u8>);

impl Handler for CollectorMemFile {
    fn write(&mut self, data_stream: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data_stream);
        Ok(data_stream.len())
    }
}

/// A gitlab repo is hosted on gitlab.com or self-hosted.
#[allow(unused)]
fn build_gitlab_url(dns_name: &str, user_name: &str, repo_name: &str, branch_name: &str) -> String {
    format!(
        "https://{}/{}/{}/-/master/{}-{}.zip",
        dns_name, user_name, repo_name, repo_name, branch_name
    )
}

#[derive(Debug)]
struct StringDeserialize(String);

#[derive(Clone, Debug)]
struct Git<'a> {
    //url: &'a str,
    zip_url: String,
    //user_name: &'a str,
    repo_name: &'a str,
    branch_name: &'a str,
}

fn check_split_url_length(url: &Vec<&str>, lenght: usize) {
    panic::set_hook(Box::new(|_| {
        println!("Url is malformed.");
    }));

    if url.len() != lenght {
        panic!()
    }
}

fn handle_git_extenstion(repo_name: &str) -> &str {
    if repo_name.ends_with(".git") {
        let repo_name = repo_name.split('.').collect::<Vec<&str>>()[0];
        repo_name
    } else {
        repo_name
    }
}

fn check_repo_data(user_name: &str, repo_name: &str) {
    panic::set_hook(Box::new(|_| {
        println!("Can't get user_namename and repository name");
    }));

    assert_ne!(user_name, "");
    assert_ne!(repo_name, "");

    let _ = panic::take_hook();
}

fn check_url_availability(response_code: u32) {
    panic::set_hook(Box::new(|_| {
        println!("This branch name doesn't exist or repo can't be fetched");
    }));

    assert_eq!(response_code, 200);

    let _ = panic::take_hook();
}

fn handle_zip_extension(file_name: &str) -> String {
    if file_name.ends_with(".zip") {
        file_name.to_string()
    } else {
        format!("{}.zip", file_name)
    }
}

impl<'a> Git<'a> {
    fn deserialize_input_url(url: &str) -> (&str, &str) {
        if url.starts_with("http") {
            let url_split: Vec<&str> = url.split("://").collect();
            let repo_url: Vec<&str> = url_split[1].split_terminator('/').collect();

            check_split_url_length(&repo_url, 3);

            let user_name: &UserName = repo_url[1];
            let repo_name: &RepoName = handle_git_extenstion(repo_url[2]);

            (user_name, repo_name)
        } else if url.starts_with("github.com") {
            let repo_url: Vec<&str> = url.split_terminator('/').collect();
            check_split_url_length(&repo_url, 3);

            let user_name: &UserName = repo_url[1];
            let repo_name: &RepoName = handle_git_extenstion(repo_url[2]);

            (user_name, repo_name)
        } else if !url.starts_with("http") && url.contains(':') {
            let repo_data: Vec<&str> = url.split(':').collect();
            check_split_url_length(&repo_data, 2);

            let user_name: &UserName = repo_data[0];
            let repo_name: &RepoName = repo_data[1];
            check_repo_data(user_name, repo_name);

            (user_name, repo_name)
        } else {
            panic!()
        }
    }

    fn build_github_url(user_name: &str, repo_name: &str, branch_name: &str) -> String {
        let mut zip_url = String::from("https://codeload.github.com");
        zip_url.push_str(format!("/{}", user_name).as_str());
        zip_url.push_str(format!("/{}", repo_name).as_str());
        zip_url.push_str(format!("/zip/refs/heads/{}", branch_name).as_str());
        zip_url
    }

    pub fn new(url: &'a str, optional_branch_name: Option<&'a String>) -> Self {
        let (user_name, repo_name) = Self::deserialize_input_url(url);

        let branch_name = match optional_branch_name {
            Some(branch) => branch,
            None => "main",
        };

        let zip_url = Self::build_github_url(user_name, repo_name, branch_name);

        let response: Vec<u8> = Vec::new();
        let mut easy = Easy2::new(CollectorMemFile(response));
        easy.get(true).unwrap();
        easy.url(&zip_url).unwrap();
        easy.perform().unwrap();

        check_url_availability(easy.response_code().unwrap());

        Self {
            zip_url,
            repo_name,
            branch_name,
        }
    }

    pub fn curl_in_memory(&self) -> Vec<u8> {
        let memfile: Vec<u8> = Vec::new();
        let mut easy = Easy2::new(CollectorMemFile(memfile));

        easy.get(true).unwrap();
        easy.url(&self.zip_url).unwrap();
        easy.perform().unwrap();
        assert_eq!(easy.response_code().unwrap(), 200);

        let mut buffer: Vec<u8> = Vec::new();
        let mut data_stream = easy.get_ref().0.as_slice();
        data_stream.read_to_end(&mut buffer).unwrap();

        buffer
    }

    pub fn extract_zip(&self, raw_compressed_data: Vec<u8>, optional_file_name: Option<&String>) {
        let data_stream = Cursor::new(raw_compressed_data);
        let mut zip_archive = zip::ZipArchive::new(data_stream).unwrap();

        let file_name = match optional_file_name {
            Some(filename) => filename,
            None => self.branch_name,
        };

        //println!("extracting to {}", file_name);
        zip_archive.extract(file_name).unwrap();

        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;

        let source_dir = format!("{}/{}-{}", file_name, self.repo_name, self.branch_name);

        //println!("Moving dir from {} to {}", source_dir, file_name);
        move_dir(source_dir, "tmp", &options).unwrap();
        fs::remove_dir_all(file_name).unwrap();
        fs::rename("tmp", file_name).unwrap();
    }

    fn save_zip(&self, raw_compressed_data: Vec<u8>, optional_file_name: Option<&String>) {
        // Unused if Some(_)
        let default_zip_name = format!("{}.zip", self.branch_name);
        let file_name = match optional_file_name {
            Some(filename) => filename,
            None => &default_zip_name,
        };

        let file_name = handle_zip_extension(file_name);
        let mut fd = File::create(file_name).unwrap();

        fd.write_all(&raw_compressed_data).unwrap();
    }
}

fn main() {
    let matches = App::new("cgit")
        .version(crate_version!())
        .author("Shadawck")
        .about("Clone git repository with curl")
        .arg(
            Arg::new("url")
                .value_name("URL")
                .help("Github link or just <user_name_name>:<repo_name>")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_parser(value_parser!(String))
                .index(1)
                .required(true),
        )
        .arg(
            Arg::new("branch")
                .short('b')
                .long("branch")
                .help("Clone a specific branch of git repositiry")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("zip")
                .short('z')
                .long("only-zip")
                .help("Only fetch the zipfile of the git repository without decompressing")
                .takes_value(false),
        )
        .arg(
            Arg::new("output")
                .value_name("PATH")
                .short('o')
                .long("output")
                .help("Path to save or decompress the zip archive")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_parser(value_parser!(String)),
        )
        .get_matches();

    let pre_url: String = matches.get_one::<String>("url").unwrap().to_string();
    let url: &str = pre_url.as_str();

    let branch_name: Option<&String> = matches.get_one("branch");

    let git = Git::new(url, branch_name);
    let raw_compressed_data = git.curl_in_memory();

    let optional_file_name: Option<&String> = matches.get_one("output");

    if matches.is_present("zip") {
        git.save_zip(raw_compressed_data, optional_file_name)
    } else {
        git.extract_zip(raw_compressed_data, optional_file_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GITHUB_URL: &str = "https://codeload.github.com/shadawck/rust-trend/zip/refs/heads/main";

    #[test]
    fn with_full_repo_url_when_deserialize_input_url_then_construct_valid_url() {
        let full_repo_url = "https://github.com/shadawck/rust-trend/".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&full_repo_url, Some(&optional_branch_name));

        assert_eq!(git.zip_url, GITHUB_URL)
    }

    #[test]
    fn with_repo_url_without_http_scheme_when_deserialize_input_url_then_construct_valid_url() {
        let url_without_scheme = "github.com/shadawck/rust-trend".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&url_without_scheme, Some(&optional_branch_name));

        assert_eq!(git.zip_url, GITHUB_URL)
    }

    #[test]
    fn with_repo_data_when_deserialize_input_url_then_construct_valid_url() {
        let repo_data = "shadawck:rust-trend".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&repo_data, Some(&optional_branch_name));

        assert_eq!(git.zip_url, GITHUB_URL)
    }

    #[test]
    #[should_panic]
    fn with_repo_url_without_user_name_when_deserialize_input_url_then_fail() {
        let repo_data = "https://github.com/rust-trend".to_string();
        let optional_branch_name = "main".to_string();
        Git::new(&repo_data, Some(&optional_branch_name));
    }

    #[test]
    #[should_panic]
    fn with_repo_url_without_repo_name_when_deserialize_input_url_then_fail() {
        let repo_data = "https://github.com/shadawck".to_string();
        let optional_branch_name = "main".to_string();
        Git::new(&repo_data, Some(&optional_branch_name));
    }

    #[test]
    #[should_panic]
    fn with_repo_url_github_com_when_deserialize_input_url_then_fail() {
        let repo_data = "https://shadawck/rust-trend".to_string();
        let optional_branch_name = "main".to_string();
        Git::new(&repo_data, Some(&optional_branch_name));
    }

    #[test]
    #[should_panic]
    fn with_repo_data_without_user_name_when_deserialize_input_url_then_fail() {
        let repo_data = ":rust-trend".to_string();
        let optional_branch_name = "main".to_string();

        Git::new(&repo_data, Some(&optional_branch_name));
    }

    #[test]
    #[should_panic]
    fn with_repo_data_without_repo_name_when_deserialize_input_url_then_fail() {
        let repo_data = "shadawck:".to_string();
        let optional_branch_name = "main".to_string();

        Git::new(&repo_data, Some(&optional_branch_name));
    }
}

//struct CollectorFile(File);
//impl Handler for CollectorFile {
//    fn write(&mut self, data_stream: &[u8]) -> Result<usize, WriteError> {
//        self.0.write_all(data_stream).unwrap();
//        Ok(data_stream.len())
//    }
//}

//#[allow(unused)]
//fn curl_in_file(url: String) {
//    let fd = File::create("main.zip").unwrap();
//    let mut easy = Easy2::new(CollectorFile(fd));
//
//    easy.get(true).unwrap();
//    easy.url(&url).unwrap();
//    easy.perform().unwrap();
//    assert_eq!(easy.response_code().unwrap(), 200);
//
//    let fd = File::open("main.zip").expect("file error reading");
//    let buf_reader = BufReader::new(fd);
//    //for bytes in buf_reader.bytes() {
//    //    print!("{}", bytes.unwrap())
//    //}
//}
