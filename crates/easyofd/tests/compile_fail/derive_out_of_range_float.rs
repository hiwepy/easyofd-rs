use easyofd::OfdModel;

#[derive(OfdModel)]
struct OutOfRangeWeight {
    #[ofd(x = 0.0, y = 0.0, weight = -1.0)]
    text: String,
}

fn main() {}
