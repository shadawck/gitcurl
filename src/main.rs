use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::panic::{self};

use clap::{crate_version, value_parser, App, Arg};
use fs_extra::dir::{move_dir, CopyOptions};

// Helper Type
type HostName = str;
type UserName = str;
type RepoName = str;

static GITHUB_HOST: &str = "github";
static GITLAB_HOST: &str = "gitlab";
static GITHUB_CODELOAD: &str = "https://codeload.github.com";
static GITHUB_DEFAULT_BRANCH: &str = "main";
static GITLAB_DEFAULT_BRANCH: &str = "master";

struct Git<'a> {
    git_zip_url: String,
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

    let _ = panic::take_hook();
}

fn handle_git_extenstion(repo_name: &str) -> &str {
    if repo_name.ends_with(".git") {
        repo_name.split('.').collect::<Vec<&str>>()[0]
    } else {
        repo_name
    }
}

fn check_repo_data(host_name: &HostName, user_name: &UserName, repo_name: &RepoName) {
    panic::set_hook(Box::new(|_| {
        println!("Can't get user_namename and repository name");
    }));

    assert_ne!(host_name, "");
    assert_ne!(user_name, "");
    assert_ne!(repo_name, "");

    let _ = panic::take_hook();
}

fn check_url_availability(response_code: u32) {
    panic::set_hook(Box::new(|_| {
        println!("This repo/branch doesn't exist or repo can't be fetched");
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
    fn deserialize_input_url(url: &str) -> (&HostName, &UserName, &RepoName) {
        if url.starts_with("http") {
            let url_split: Vec<&str> = url.split("://").collect();
            let repo_url: Vec<&str> = url_split[1].split_terminator('/').collect();

            check_split_url_length(&repo_url, 3);

            let host_name: &HostName = repo_url[0];
            let user_name: &UserName = repo_url[1];
            let repo_name: &RepoName = handle_git_extenstion(repo_url[2]);

            (host_name, user_name, repo_name)
        } else if !url.starts_with("http") && !url.contains(':') {
            let repo_url: Vec<&str> = url.split_terminator('/').collect();
            check_split_url_length(&repo_url, 3);

            let host_name: &HostName = repo_url[0];
            let user_name: &UserName = repo_url[1];
            let repo_name: &RepoName = handle_git_extenstion(repo_url[2]);

            (host_name, user_name, repo_name)
        } else if !url.starts_with("http") && url.contains(':') {
            let repo_data: Vec<&str> = url.split(':').collect();
            check_split_url_length(&repo_data, 3);

            let host_name: &HostName = repo_data[0];
            let user_name: &UserName = repo_data[1];
            let repo_name: &RepoName = repo_data[2];
            check_repo_data(host_name, user_name, repo_name);

            (host_name, user_name, repo_name)
        } else {
            panic!()
        }
    }

    fn build_github_url(user_name: &str, repo_name: &str, branch_name: &str) -> String {
        format!(
            "{}/{}/{}/zip/refs/heads/{}",
            GITHUB_CODELOAD, user_name, repo_name, branch_name
        )
    }

    fn build_gitlab_url(
        host_name: &str,
        user_name: &str,
        repo_name: &str,
        branch_name: &str,
    ) -> String {
        format!(
            "https://{}/{}/{}/-/archive/{}/{}-{}.zip",
            host_name, user_name, repo_name, branch_name, repo_name, branch_name
        )
    }

    fn build_url(
        host_name: &HostName,
        user_name: &UserName,
        repo_name: &RepoName,
        branch_name: &str,
    ) -> String {
        if host_name.starts_with(GITHUB_HOST) {
            Self::build_github_url(user_name, repo_name, branch_name)
        } else {
            Self::build_gitlab_url(host_name, user_name, repo_name, branch_name)
        }
    }

    fn request_url(git_zip_url: &str) {
        panic::set_hook(Box::new(|_| {
            println!("Could not resolve host !");
        }));

        let resp = ureq::get(git_zip_url).call().unwrap();
        let _ = panic::take_hook();

        check_url_availability(resp.status().into());
    }

    fn handle_hostname(host_name: &HostName) -> &str {
        if host_name == GITLAB_HOST {
            "gitlab.com"
        } else {
            host_name
        }
    }

    pub fn new(url: &'a str, optional_branch_name: Option<&'a String>) -> Self {
        let (host_name, user_name, repo_name) = Self::deserialize_input_url(url);

        let branch_name = match optional_branch_name {
            Some(branch) => branch,
            None => {
                if host_name.starts_with(GITHUB_HOST) {
                    GITHUB_DEFAULT_BRANCH
                } else {
                    GITLAB_DEFAULT_BRANCH
                }
            }
        };

        let host_name = Self::handle_hostname(host_name);

        let git_zip_url = Self::build_url(host_name, user_name, repo_name, branch_name);

        Self::request_url(&git_zip_url);

        Self {
            git_zip_url,
            repo_name,
            branch_name,
        }
    }

    pub fn curl_in_memory(&self) -> Vec<u8> {
        let resp = ureq::get(&self.git_zip_url).call().unwrap();

        assert_eq!(resp.status(), 200);
        let mut buffer: Vec<u8> = Vec::new();

        resp.into_reader().read_to_end(&mut buffer).unwrap();

        buffer
    }

    pub fn extract_zip(&self, raw_compressed_data: Vec<u8>, optional_file_name: Option<&String>) {
        let data_stream = Cursor::new(raw_compressed_data);
        let mut zip_archive = zip::ZipArchive::new(data_stream).unwrap();

        let file_name = match optional_file_name {
            Some(filename) => filename.to_string(),
            None => format!("{}_{}", self.repo_name, self.branch_name),
        };

        println!("Extracting archive to : {}", &file_name);
        zip_archive.extract(&file_name).unwrap();

        let mut options = CopyOptions::new();
        options.overwrite = true;
        options.copy_inside = true;

        let source_dir = format!("{}/{}-{}", &file_name, self.repo_name, self.branch_name);

        move_dir(source_dir, "tmp", &options).unwrap();
        fs::remove_dir_all(&file_name).unwrap();
        fs::rename("tmp", &file_name).unwrap();
    }

    fn save_zip(&self, raw_compressed_data: Vec<u8>, optional_file_name: Option<&String>) {
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
                .help("Github | Gitlab link or just <host>:<user_name_name>:<repo_name>")
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

    let url: &str = matches.get_one::<String>("url").unwrap().as_str();
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

    const GITHUB_URL: &str = "https://codeload.github.com/shadawck/gitcurl/zip/refs/heads/main";
    const GITLAB_URL: &str =
        "https://gitlab.com/Fract/sample-project/-/archive/master/sample-project-master.zip";
    const CUSTOM_GITLAB_URL: &str =
        "https://gitlab.kitware.com/utils/rust-gitlab/-/archive/master/rust-gitlab-master.zip";

    #[test]
    fn with_github_full_repo_url_when_deserialize_input_url_then_construct_valid_url() {
        let full_repo_url = "https://github.com/shadawck/gitcurl".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&full_repo_url, Some(&optional_branch_name));

        assert_eq!(git.git_zip_url, GITHUB_URL)
    }

    #[test]
    fn with_github_full_repo_url_and_git_extension_when_deserialize_input_url_then_construct_valid_url(
    ) {
        let full_repo_url = "https://github.com/shadawck/gitcurl.git".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&full_repo_url, Some(&optional_branch_name));

        assert_eq!(git.git_zip_url, GITHUB_URL)
    }

    #[test]
    fn with_github_repo_url_without_http_scheme_when_deserialize_input_url_then_construct_valid_url(
    ) {
        let url_without_scheme = "github.com/shadawck/gitcurl".to_string();
        let optional_branch_name = "main".to_string();
        let git = Git::new(&url_without_scheme, Some(&optional_branch_name));

        assert_eq!(git.git_zip_url, GITHUB_URL)
    }

    #[test]
    fn with_github_repo_data_when_deserialize_input_url_then_construct_valid_url() {
        let repo_data = "github:shadawck:gitcurl".to_string();
        let git = Git::new(&repo_data, None);

        assert_eq!(git.git_zip_url, GITHUB_URL)
    }

    #[test]
    fn with_gitlab_full_repo_url_when_deserialize_input_url_then_construct_valid_url() {
        let full_repo_url = "https://gitlab.com/Fract/sample-project".to_string();
        let git = Git::new(&full_repo_url, None);

        assert_eq!(git.git_zip_url, GITLAB_URL)
    }

    #[test]
    fn with_gitlab_repo_url_without_http_scheme_when_deserialize_input_url_then_construct_valid_url(
    ) {
        let url_without_scheme = "gitlab.com/Fract/sample-project".to_string();
        let git = Git::new(&url_without_scheme, None);

        assert_eq!(git.git_zip_url, GITLAB_URL)
    }

    #[test]
    fn with_gitlab_repo_data_when_deserialize_input_url_then_construct_valid_url() {
        let repo_data = "gitlab:Fract:sample-project".to_string();
        let git = Git::new(&repo_data, None);

        assert_eq!(git.git_zip_url, GITLAB_URL)
    }

    #[test]
    fn with_on_premise_gitlab_full_repo_url_when_deserialize_input_url_then_construct_valid_url() {
        let full_repo_url = "https://gitlab.kitware.com/utils/rust-gitlab".to_string();
        let git = Git::new(&full_repo_url, None);

        assert_eq!(git.git_zip_url, CUSTOM_GITLAB_URL)
    }

    #[test]
    fn with_with_on_premise_gitlab_repo_url_without_http_scheme_when_deserialize_input_url_then_construct_valid_url(
    ) {
        let url_without_scheme = "gitlab.kitware.com/utils/rust-gitlab".to_string();
        let git = Git::new(&url_without_scheme, None);

        assert_eq!(git.git_zip_url, CUSTOM_GITLAB_URL)
    }

    #[test]
    fn with_with_on_premise_gitlab_repo_data_when_deserialize_input_url_then_construct_valid_url() {
        let repo_data = "gitlab.kitware.com:utils:rust-gitlab".to_string();
        let git = Git::new(&repo_data, None);

        assert_eq!(git.git_zip_url, CUSTOM_GITLAB_URL)
    }

    #[test]
    #[should_panic]
    fn with_repo_url_without_user_name_when_deserialize_input_url_then_fail() {
        let repo_data = "https://github.com/gitcurl".to_string();
        Git::new(&repo_data, None);
    }

    #[test]
    #[should_panic]
    fn with_repo_url_without_repo_name_when_deserialize_input_url_then_fail() {
        let repo_data = "https://github.com/shadawck".to_string();
        Git::new(&repo_data, None);
    }

    #[test]
    #[should_panic]
    fn with_repo_url_without_host_when_deserialize_input_url_then_fail() {
        let repo_data = "https://shadawck/gitcurl".to_string();
        Git::new(&repo_data, None);
    }

    #[test]
    #[should_panic]
    fn with_repo_data_without_user_name_when_deserialize_input_url_then_fail() {
        let repo_data = ":gitcurl".to_string();
        Git::new(&repo_data, None);
    }

    #[test]
    #[should_panic]
    fn with_repo_data_without_repo_name_when_deserialize_input_url_then_fail() {
        let repo_data = "shadawck:".to_string();
        Git::new(&repo_data, None);
    }

    #[test]
    fn with_valid_repo_when_curl_in_memory_then_buffer_not_empty() {
        let repo_data = "https://github.com/shadawck/gitcurl".to_string();
        let buffer = Git::new(&repo_data, None).curl_in_memory();
        assert!(!buffer.is_empty())
    }

    use std::path::PathBuf;
    #[test]
    fn with_valid_repo_when_curl_in_memory_then_buffer_can_be_saved_as_zip() {
        let repo_data = "https://github.com/shadawck/gitcurl".to_string();
        let git = Git::new(&repo_data, None);
        let buffer = git.curl_in_memory();

        git.save_zip(buffer, None);
        let zip = PathBuf::from("main.zip");
        assert!(zip.is_file());
    }

    use std::io;
    #[test]
    fn with_valid_repo_when_curl_in_memory_then_zip_can_be_extracted() {
        let repo_data = "https://github.com/shadawck/gitcurl".to_string();
        let git = Git::new(&repo_data, None);
        let buffer = git.curl_in_memory();

        git.extract_zip(buffer, None);
        let extracted_archvive = PathBuf::from("gitcurl_main");
        assert!(extracted_archvive.is_dir());

        let entries = fs::read_dir(extracted_archvive)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();
        assert!(!entries.is_empty())
    }
}
