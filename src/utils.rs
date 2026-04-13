// NOTE maybe let parsing return enums comands varients
pub fn parsing(detailed_msg: &String) -> (String, String) {
    if !detailed_msg.is_empty() {
        let detailed_msg: Vec<&str> = detailed_msg.split(':').collect();

        if let [msg, name] = detailed_msg[..] {
            //dbg!(&msg); dbg!(&name);
            return ((String::from(name)), String::from(msg));
        } else {
            unreachable!();
        }
    } else {
        return ((String::new()), (String::new()));
    }
}
