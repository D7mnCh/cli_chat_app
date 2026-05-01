use crate::app::NameValidation;

// NOTE look here of entering name problem
pub fn parsing_name_server(name: Option<String>, clients_name: Vec<&String>) -> NameValidation {
    if let Some(name) = name {
        for other_client_name in clients_name {
            if name == *other_client_name {
                return NameValidation::Used;
            } else if name == "Server" || name == "server" {
                return NameValidation::Reserved;
            } else if name.is_empty() {
                return NameValidation::Empty;
            }
        }
        return NameValidation::Valid(name);
    }
    return NameValidation::Empty;
}
