use {
    phf::{phf_set, Set},
};

static IMAGE_EXTENSIONS: Set<&'static str> = phf_set! {
    "jpg", "JPG",
    "jpeg", "JPEG",
    "png", "PNG",
};

pub fn is_image(ext: &str) -> bool {
    IMAGE_EXTENSIONS.contains(ext)
}
