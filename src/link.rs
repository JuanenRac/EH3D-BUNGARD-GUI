//! The connection of the machine as the classic programs keep it, in one small file (`machine.cfg`): what the Connection window writes
//! into it, without touching any other line (the tuning of the machine stays as it is).

use eh3d_drillpro::protocol::connection::{ConnectionConfig, Kind};

/// The keys the Connection window owns, with the value each has for this connection.
pub fn pairs(c: &ConnectionConfig) -> Vec<(&'static str, String)> {
    let mut v = vec![
        ("connection", c.kind.name().to_string()),
        ("rs232_port", c.rs232_port.clone()),
        ("usb_port", c.usb_port.clone()),
    ];
    if matches!(c.kind, Kind::Node) || !c.node_address.is_empty() {
        v.push(("node_address", c.node_address.clone()));
        v.push(("node_token", c.node_token.clone()));
    }
    if matches!(c.kind, Kind::Server) || !c.server_address.is_empty() {
        v.push(("server_address", c.server_address.clone()));
        v.push(("server_token", c.server_token.clone()));
        v.push(("server_machine", c.server_machine.clone()));
    }
    v
}

/// The text of the file with the keys of `pairs` set: a line that has the key is rewritten, the others are left as they are, and a key
/// that was not there is added at the end.
pub fn update(old: &str, pairs: &[(&str, String)]) -> String {
    let mut done = vec![false; pairs.len()];
    let mut out = Vec::new();
    for line in old.lines() {
        let t = line.trim();
        let key = (!t.starts_with('#') && !t.starts_with(';'))
            .then(|| t.split_once('='))
            .flatten()
            .map(|(k, _)| k.trim().to_ascii_lowercase());
        match key.and_then(|k| pairs.iter().position(|(p, _)| *p == k)) {
            Some(i) if !done[i] => {
                done[i] = true;
                out.push(format!("{} = {}", pairs[i].0, pairs[i].1));
            }
            Some(_) => {}
            None => out.push(line.to_string()),
        }
    }
    for (i, (k, v)) in pairs.iter().enumerate() {
        if !done[i] {
            out.push(format!("{k} = {v}"));
        }
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use eh3d_drillpro::protocol::connection::parse;

    #[test]
    fn the_keys_are_set_and_the_rest_is_kept() {
        let old = "# my machine\nconnection = demo\nx_search_hz = 2700\nusb_port = COM3\n";
        let mut c = parse(old).unwrap();
        c.kind = Kind::Usb;
        c.usb_port = "COM8".into();
        let text = update(old, &pairs(&c));
        assert!(text.contains("# my machine"));
        assert!(text.contains("x_search_hz = 2700"));
        assert!(text.contains("connection = usb"));
        assert!(text.contains("usb_port = COM8"));
        let back = parse(&text).unwrap();
        assert_eq!((back.kind, back.usb_port.as_str()), (Kind::Usb, "COM8"));
        assert_eq!(back.tuning, c.tuning);
    }

    #[test]
    fn the_node_and_the_server_are_written_only_when_they_are_used() {
        let mut c = ConnectionConfig::default();
        assert!(!update("", &pairs(&c)).contains("node_address"));
        c.kind = Kind::Server;
        c.server_address = "192.168.0.10:8080".into();
        c.server_machine = "bungard-1".into();
        let back = parse(&update("", &pairs(&c))).unwrap();
        assert_eq!(back.server_address, "192.168.0.10:8080");
        assert_eq!(back.server_machine, "bungard-1");
    }
}
