//! `bungard-gui --server-status` and `bungard-gui --machines`: what the EH3D server says, written to the terminal with no window. Both read the link from the machine
//! file the classic programs read (`--connection FILE`, or the default one): `server_address`, `server_token`, `server_machine` and the limits of the link
//! (`server_timeout_s`, `server_retries`, `server_mdns`, `server_max_answer_mib`), and ask through the client of the suite (`eh3d-serverclient` of EH3D-COMMON): a
//! time limit and retries with growing pauses on each request, and the refusals told by their code in the language of the system.

use std::path::{Path, PathBuf};

use eh3d_fleet::Fleet;
use eh3d_serverclient as sc;

use crate::server_wire;

/// Why a command could not give its answer, and the code the program leaves with: 2 when the machine file has no server to ask, 1 when the server did not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub code: i32,
    pub text: String,
}

/// The machine file: the one named, or the default one of the user.
pub fn machine_file(connection: Option<&Path>) -> PathBuf {
    connection
        .map(Path::to_path_buf)
        .unwrap_or_else(eh3d_drillpro::protocol::connection::default_path)
}

/// The settings of the link in the text of a machine file; the keys of the classic programs are left alone. A file with no `server_address` has no server to ask.
pub fn settings_from_text(text: &str) -> Result<sc::Settings, String> {
    let settings = sc::Settings::from_text(text)?;
    if !settings.is_configured() {
        return Err("the machine file has no server_address: there is no server to ask".into());
    }
    Ok(settings)
}

/// The settings of the link in a machine file.
pub fn settings_of(connection: Option<&Path>) -> Result<sc::Settings, String> {
    let file = machine_file(connection);
    let text = std::fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    settings_from_text(&text).map_err(|e| format!("{}: {e}", file.display()))
}

/// What a refusal of the server says, as one line.
fn refusal_line(status: u16, raw: &str) -> String {
    let body: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::Value::Null);
    match (body["error"].as_str(), body["detail"].as_str()) {
        (Some(code), Some(detail)) if !detail.is_empty() => format!("{code}: {detail}"),
        (Some(code), _) => code.to_string(),
        _ => format!("the server answered {status}"),
    }
}

/// The machines and the stations of the server, one line each, and the count at the end; the answer of `GET /api/v1/fleet` read over this client.
pub fn machines_from(client: &mut sc::Client, lang: sc::Lang) -> Result<String, String> {
    let (status, raw) = server_wire::exchange(client, "GET", "/api/v1/fleet", None, lang)?;
    if status != 200 {
        return Err(refusal_line(status, &raw));
    }
    let value: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("the answer of the server is not JSON: {e}"))?;
    let fleet = Fleet::from_json(&value)?;
    let s = fleet.summary();
    Ok(format!(
        "{}{} machines: {} free, {} busy, {} waiting for the operator; {} stations\n",
        fleet.to_text(),
        s.machines,
        s.idle,
        s.busy,
        s.waiting,
        s.stations
    ))
}

/// `--server-status`: what the server answers at `/health` and the metrics of the requests of this run.
pub fn status(connection: Option<&Path>) -> Result<String, Failure> {
    let settings = settings_of(connection).map_err(|text| Failure { code: 2, text })?;
    let mut client = sc::Client::connect(settings);
    server_wire::status_text(&mut client, server_wire::language())
        .map_err(|text| Failure { code: 1, text })
}

/// `--machines`: every machine of the server in its station.
pub fn machines(connection: Option<&Path>) -> Result<String, Failure> {
    let settings = settings_of(connection).map_err(|text| Failure { code: 2, text })?;
    let mut client = sc::Client::connect(settings);
    machines_from(&mut client, server_wire::language()).map_err(|text| Failure { code: 1, text })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sc::{Reply, Request, Transport, TransportError};

    /// A server that answers the fleet and nothing else.
    struct Canned(String);

    impl Transport for Canned {
        fn send(&self, r: &Request) -> Result<Reply, TransportError> {
            if r.path == "/api/v1/fleet" {
                Ok(Reply {
                    status: 200,
                    body: self.0.clone(),
                })
            } else {
                Ok(Reply {
                    status: 404,
                    body: "{\"error\":\"not_found\",\"detail\":\"\"}".into(),
                })
            }
        }
    }

    const FLEET: &str = r#"{"stations":[{"id":"bench","machines":["m1","m2"]}],"machines":[
        {"id":"m1","name":"M1","station":"bench","connection":"demo","state":"busy","job":{"id":"j","name":"board","kind":"drill","items_done":1,"items_total":4,"percent":25},"ready":0},
        {"id":"m2","name":"M2","station":"bench","connection":"demo","state":"idle","job":null,"ready":2}]}"#;

    fn client_over(transport: Box<dyn Transport>) -> sc::Client {
        let settings =
            sc::Settings::from_text("server_address = 127.0.0.1:1\nserver_token = t\n").unwrap();
        sc::Client::new(settings, transport)
    }

    #[test]
    fn the_link_is_read_from_the_machine_file_and_the_classic_keys_are_left_alone() {
        let s = settings_from_text(
            "# my machine\nconnection = server\nx_search_hz = 2700\nserver_address = 10.0.0.5:18200\nserver_token = abc\nserver_machine = bench\nserver_timeout_s = 30\nserver_retries = 4\nserver_mdns = off\n",
        )
        .unwrap();
        assert_eq!(s.server_address, "10.0.0.5:18200");
        assert_eq!(s.server_machine, "bench");
        assert_eq!((s.server_timeout_s, s.server_retries), (30, 4));
        // the defaults of the limits when the file does not say
        let d = settings_from_text("server_address = a:1\n").unwrap();
        assert_eq!(
            (d.server_timeout_s, d.server_retries, d.server_mdns),
            (15, 2, false)
        );
        assert_eq!(d.server_max_answer_mib, 8);
    }

    #[test]
    fn a_file_without_a_server_or_with_a_value_out_of_its_range_is_refused_with_the_reason() {
        let e = settings_from_text("connection = demo\n").unwrap_err();
        assert!(e.contains("no server_address"), "{e}");
        let e = settings_from_text("server_address = a:1\nserver_retries = 9\n").unwrap_err();
        assert!(e.contains("server_retries") && e.contains("line 2"), "{e}");
        let e = settings_from_text("server_address = a:1\nserver_timeout_s = 500\n").unwrap_err();
        assert!(e.contains("server_timeout_s"), "{e}");
        let dir = std::env::temp_dir().join(format!("eh3d_bg_servercmd_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("machine.cfg");
        std::fs::write(&file, "connection = demo\n").unwrap();
        let failure = status(Some(&file)).unwrap_err();
        assert_eq!(failure.code, 2);
        assert!(failure.text.contains("machine.cfg"), "{}", failure.text);
        let missing = machines(Some(&dir.join("nothing.cfg"))).unwrap_err();
        assert_eq!(missing.code, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_machines_are_listed_by_station_with_their_count() {
        let mut client = client_over(Box::new(Canned(FLEET.into())));
        let text = machines_from(&mut client, sc::Lang::En).unwrap();
        assert!(text.contains("m1") && text.contains("m2"), "{text}");
        assert!(
            text.ends_with("2 machines: 1 free, 1 busy, 0 waiting for the operator; 1 stations\n"),
            "{text}"
        );
    }

    #[test]
    fn a_refusal_of_the_server_is_one_line_and_an_answer_that_is_not_json_is_told() {
        let mut client = client_over(Box::new(sc::sim::SimServer::new()));
        let e = machines_from(&mut client, sc::Lang::En).unwrap_err();
        assert_eq!(e, "not_found");
        let mut client = client_over(Box::new(Canned("not json".into())));
        let e = machines_from(&mut client, sc::Lang::En).unwrap_err();
        assert!(e.contains("not JSON"), "{e}");
        assert_eq!(
            refusal_line(409, r#"{"error":"busy","detail":"a job is running"}"#),
            "busy: a job is running"
        );
        assert_eq!(refusal_line(502, ""), "the server answered 502");
    }
}
