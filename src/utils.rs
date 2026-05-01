use crate::app::NameValidation;

pub fn parsing_name_server(name: Option<String>, clients_name: Vec<&String>) -> NameValidation {
    let name = match name {
        Some(name) => name,
        None => return NameValidation::Empty,
    };

    if name.eq_ignore_ascii_case("server") {
        return NameValidation::Reserved;
    } else if name.is_empty() {
        return NameValidation::Empty;
    }

    for other_client_name in clients_name.iter() {
        if name == **other_client_name {
            return NameValidation::Used;
        }
    }

    return NameValidation::Valid(name);
}
