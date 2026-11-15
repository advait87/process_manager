use rand::Rng;
use sha2::{ Digest, Sha256 };
use std::path::Path;

pub fn get_random_sha256() -> [u8; 32] {
    let mut bytes = [0u8; 32];
    rand::fill(&mut bytes);

    Sha256::digest(bytes).into()
}

pub fn python_docker_file_string(file_path: &Path) -> String {
    format!(r##"
FROM python:3

WORKDIR /usr/src/app

COPY . .
EXPOSE 3000
CMD ["python3", "python_code.py"]
        "##,
        // file_path.to_str().unwrap().to_string(),
        // file_path.to_str().unwrap().to_string()
    )
    .to_string()

}

