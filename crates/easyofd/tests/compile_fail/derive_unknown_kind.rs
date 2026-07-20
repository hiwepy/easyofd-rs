use easyofd::OfdModel;

#[derive(OfdModel)]
struct BadKind {
    #[ofd(x = 0.0, y = 0.0, kind = "table")]
    field: String,
}

fn main() {}
