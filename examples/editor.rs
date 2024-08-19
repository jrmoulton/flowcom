#[tokio::main]
pub async fn main() {
    floem::launch(move || flowcom::helix::test());
}
