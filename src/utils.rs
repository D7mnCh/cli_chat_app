pub fn parsing(detailed_msg: &String) -> (String, String) {
    let detailed_msg: Vec<&str> = detailed_msg.split(':').collect();
    if let [msg, name] = detailed_msg[..] {
        return ((String::from(name)), String::from(msg));
    } else {
        dbg!(detailed_msg);
        unreachable!();
    }
}
