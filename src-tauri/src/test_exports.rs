use sherpa_rs::*;

fn main() {
    // This will error and show me what's available if I use an invalid name
    let _ = sherpa_rs::sensevoice::SenseVoiceRecognizer::new();
}
