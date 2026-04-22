pub enum NameHandling {
    Empty,
    Reserved,
    Valid,
}

pub fn parsing_name(name: &str) -> NameHandling {
    if name.is_empty() {
        return NameHandling::Empty;
    } else if name == "Server" || name == "server" {
        return NameHandling::Reserved;
    } else {
        return NameHandling::Valid;
    }
}
