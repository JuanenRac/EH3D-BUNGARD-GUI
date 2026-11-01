//! The window of the node of the drill head (EH3D-NODE-BUNGARD-AUX): how it is reached (not used, directly or through the EH3D server), its address and token, the
//! picture of its camera when it is switched on, the speed and the temperature of the spindle and a measure of the table under the head. The node is asked from other
//! threads, so a node that does not answer never holds the window.

use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;

use eh3d_aux::{Heat, Link, Measure, MjpegSplitter, Status};
use eh3d_drillpro::protocol::connection::HeadNode;

use super::*;

/// How often the status is asked.
const POLL: Duration = Duration::from_secs(2);

type Frame = (usize, usize, Vec<u8>);

#[derive(Default)]
pub(crate) struct HeadState {
    pub status: Option<Result<Status, String>>,
    status_rx: Option<Receiver<Result<Status, String>>>,
    pub(super) polled: Option<Instant>,
    pub measure: Option<Result<Measure, String>>,
    measure_rx: Option<Receiver<Result<Measure, String>>>,
    frame_rx: Option<Receiver<Frame>>,
    stop_camera: Option<Arc<AtomicBool>>,
    pub texture: Option<egui::TextureHandle>,
}

/// `host` or `host:port` (the port of the node is 80 when it is not said).
fn split_address(a: &str) -> Option<(String, u16)> {
    let a = a.trim();
    if a.is_empty() {
        return None;
    }
    match a.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() => p.parse().ok().map(|p| (h.to_string(), p)),
        _ => Some((a.to_string(), 80)),
    }
}

impl App {
    /// The link to the node: directly with its own address and token, or through the server of the connection window (its address, its token and the id of the
    /// machine; the server holds the token of the node). `None` when the node is off or the data for the way chosen are missing.
    pub(crate) fn head_link(&self) -> Option<Link> {
        match self.conn.head_node {
            HeadNode::Off => None,
            HeadNode::Direct => {
                let (host, port) = split_address(&self.conn.head_address)?;
                Some(Link::new(&host, port, &self.conn.head_token))
            }
            HeadNode::Server => {
                let machine = self.conn.server_machine.trim();
                if machine.is_empty() {
                    return None;
                }
                // 18200 is the port of the EH3D server
                let (host, port) = match split_address(&self.conn.server_address) {
                    Some((h, p)) if self.conn.server_address.contains(':') => (h, p),
                    Some((h, _)) => (h, 18200),
                    None => return None,
                };
                Some(Link::via_server(
                    &host,
                    port,
                    &self.conn.server_token,
                    machine,
                ))
            }
        }
    }

    /// Asks the node for its status now.
    pub(crate) fn head_poll(&mut self) {
        let Some(link) = self.head_link() else {
            self.head.status = None;
            return;
        };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(link.status());
        });
        self.head.status_rx = Some(rx);
        self.head.polled = Some(Instant::now());
    }

    /// One measure of the table under the head.
    pub(crate) fn head_measure(&mut self) {
        let Some(link) = self.head_link() else { return };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(link.measure(0));
        });
        self.head.measure_rx = Some(rx);
        self.head.measure = None;
    }

    fn head_stop_camera(&mut self) {
        if let Some(stop) = self.head.stop_camera.take() {
            stop.store(true, Ordering::SeqCst);
        }
        self.head.frame_rx = None;
        self.head.texture = None;
    }

    fn head_start_camera(&mut self) {
        let Some(link) = self.head_link() else { return };
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel::<Frame>();
        let flag = stop.clone();
        std::thread::spawn(move || {
            while !flag.load(Ordering::SeqCst) {
                if let Ok(mut s) = link.open_stream() {
                    let mut split = MjpegSplitter::new();
                    let mut buf = [0u8; 16 * 1024];
                    while !flag.load(Ordering::SeqCst) {
                        let n = match s.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => n,
                        };
                        for jpeg in split.push(&buf[..n]) {
                            let mut d = jpeg_decoder::Decoder::new(&jpeg[..]);
                            if let (Ok(px), Some(info)) = (d.decode(), d.info()) {
                                if info.pixel_format == jpeg_decoder::PixelFormat::RGB24
                                    && tx
                                        .send((info.width as usize, info.height as usize, px))
                                        .is_err()
                                {
                                    return;
                                }
                            }
                        }
                    }
                }
                // a node that is not there yet: try again in a while
                for _ in 0..20 {
                    if flag.load(Ordering::SeqCst) {
                        return;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        });
        self.head.stop_camera = Some(stop);
        self.head.frame_rx = Some(rx);
    }

    /// Called on every frame: while the window is open the node is asked, and its camera runs only when it is switched on and the window is open.
    pub(super) fn head_tick(&mut self, ctx: &egui::Context) {
        // the camera and the status are followed while the window of the node or the one of the quality check is open
        let open = matches!(self.modal, Some(Modal::Head | Modal::Inspect));
        self.insp_tick();
        let link = self.head_link();
        if let Some(r) = self
            .head
            .status_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.head.status = Some(r);
            self.head.status_rx = None;
        }
        if let Some(r) = self
            .head
            .measure_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
        {
            self.head.measure = Some(r);
            self.head.measure_rx = None;
        }
        // the quality check needs the picture whatever the window of the node says
        let camera =
            open && link.is_some() && (self.conn.head_camera || self.modal == Some(Modal::Inspect));
        if camera && self.head.stop_camera.is_none() {
            self.head_start_camera();
        } else if !camera && self.head.stop_camera.is_some() {
            self.head_stop_camera();
        }
        if let Some(rx) = &self.head.frame_rx {
            let mut last = None;
            while let Ok(f) = rx.try_recv() {
                last = Some(f);
            }
            if let Some((w, h, rgb)) = last {
                let image = egui::ColorImage::from_rgb([w, h], &rgb);
                self.insp.picture = (w, h);
                self.head.texture =
                    Some(ctx.load_texture("head-camera", image, egui::TextureOptions::LINEAR));
            }
        }
        if open
            && link.is_some()
            && self.head.status_rx.is_none()
            && self.head.polled.is_none_or(|t| t.elapsed() >= POLL)
        {
            self.head_poll();
        }
        if open {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }

    /// The text boxes of the window, over the card.
    pub(super) fn head_fields(&mut self, ctx: &egui::Context) {
        let u = self.ui_scale;
        for i in 0..2 {
            let pos = Pos2::new(
                self.ui_origin.x + 700.0 * u,
                self.ui_origin.y + (130.0 + 150.0 + i as f32 * 58.0) * u,
            );
            let buf = if i == 0 {
                &mut self.conn.head_address
            } else {
                &mut self.conn.head_token
            };
            egui::Area::new(egui::Id::unique(("head-field", i)))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let r = super::look::text_box(
                        ui,
                        egui::TextEdit::singleline(buf)
                            .desired_width(430.0 * u)
                            .font(FontId::proportional(18.0 * u))
                            .password(i == 1),
                    );
                    if r.changed() {
                        self.saved = false;
                        self.head.status = None;
                    }
                });
        }
    }

    pub(super) fn paint_head(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::HeadTitle), 700.0);
        for (i, (mode, label)) in [
            (HeadNode::Off, Text::HeadOff),
            (HeadNode::Direct, Text::HeadDirect),
            (HeadNode::Server, Text::HeadServer),
        ]
        .into_iter()
        .enumerate()
        {
            cv.chip(
                x + 30.0 + i as f32 * 232.0,
                y + 84.0,
                220.0,
                44.0,
                self.tr(label),
                self.conn.head_node == mode,
                Act::HeadMode(mode),
            );
        }
        cv.text(
            x + 30.0,
            y + 180.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::ConnAddress),
            17.0,
            col(t.text),
        );
        cv.text(
            x + 30.0,
            y + 238.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::ConnToken),
            17.0,
            col(t.text),
        );
        cv.checkbox(
            x + 30.0,
            y + 262.0,
            680.0,
            self.tr(Text::HeadCamera),
            self.conn.head_camera,
            Act::HeadCamera,
        );
        let link = self.head_link();
        let note = match (self.conn.head_node, &link) {
            (HeadNode::Server, None) => Some((self.tr(Text::HeadNoServer), col(t.dim))),
            (HeadNode::Server, Some(_)) => Some((self.tr(Text::HeadServerNote), col(t.dim))),
            (HeadNode::Direct, None) => Some((self.tr(Text::HeadNoLink), col(t.dim))),
            _ => None,
        };
        if let Some((s, c)) = note {
            cv.text(x + 30.0, y + 330.0, Align2::LEFT_BOTTOM, s, 16.0, c);
        }
        // the camera
        let (cx, cy, cw, ch) = (x + 30.0, y + 350.0, 400.0, 270.0);
        cv.p.rect_filled(
            cv.rect(cx, cy, cw, ch),
            4.0 * cv.u,
            alpha(col(t.panel_raised), 255),
        );
        match &self.head.texture {
            Some(tex) => {
                cv.p.image(
                    tex.id(),
                    cv.rect(cx, cy, cw, ch),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                let s = if self.conn.head_camera && link.is_some() {
                    self.tr(Text::HeadWaiting)
                } else {
                    self.tr(Text::HeadCameraOff)
                };
                cv.text(
                    cx + cw / 2.0,
                    cy + ch / 2.0,
                    Align2::CENTER_CENTER,
                    s,
                    16.0,
                    col(t.dim),
                );
            }
        }
        // what the node says
        let rx = x + 450.0;
        match &self.head.status {
            Some(Ok(s)) => {
                let heat = match s.heat {
                    Heat::Ok => Text::HeatOk,
                    Heat::Warm => Text::HeatWarm,
                    Heat::Hot => Text::HeatHot,
                    Heat::Unknown => Text::HeatUnknown,
                };
                let temp = s.temp_c.map_or("-".to_string(), |c| format!("{c:.1} C"));
                let lines = [
                    (
                        if s.camera_model.is_empty() {
                            format!("{}  {}", s.node, s.version)
                        } else {
                            // the model of the camera in brackets: the line has to fit the card (the label of the model is in the title of the window of the node)
                            format!("{}  {}  ({})", s.node, s.version, s.camera_model)
                        },
                        col(t.accent),
                    ),
                    (
                        format!("{}: {:.0} rpm", self.tr(Text::HeadSpeed), s.rpm),
                        col(t.text),
                    ),
                    (
                        format!("{}: {temp}  ({})", self.tr(Text::HeadTemp), self.tr(heat)),
                        col(t.text),
                    ),
                    (
                        self.tr(if s.protect_open {
                            Text::HeadProtectOpen
                        } else {
                            Text::HeadProtectClosed
                        })
                        .to_string(),
                        if s.protect_open {
                            col(t.danger)
                        } else {
                            col(t.ok)
                        },
                    ),
                ];
                for (i, (l, c)) in lines.into_iter().enumerate() {
                    cv.text(
                        rx,
                        y + 372.0 + i as f32 * 34.0,
                        Align2::LEFT_BOTTOM,
                        &l,
                        17.0,
                        c,
                    );
                }
            }
            Some(Err(e)) => {
                let e: String = e.chars().take(60).collect();
                cv.text(rx, y + 372.0, Align2::LEFT_BOTTOM, &e, 15.0, col(t.danger));
            }
            None => {
                if link.is_some() {
                    cv.text(
                        rx,
                        y + 372.0,
                        Align2::LEFT_BOTTOM,
                        self.tr(Text::HeadWaiting),
                        16.0,
                        col(t.dim),
                    );
                }
            }
        }
        if link.is_some() {
            // the ring of LEDs around the camera
            match &self.head.status {
                Some(Ok(s)) if s.light => {
                    let on = s.light_on;
                    cv.text(
                        rx,
                        y + 496.0,
                        Align2::LEFT_BOTTOM,
                        &if on {
                            format!("{} {} %", self.tr(Text::LightOn), s.light_level)
                        } else {
                            self.tr(Text::LightOff).to_string()
                        },
                        15.0,
                        col(t.text),
                    );
                    cv.chip(rx, y + 504.0, 60.0, 34.0, "0", !on, Act::HeadLight(None));
                    for (i, level) in [30u32, 60, 80].into_iter().enumerate() {
                        cv.chip(
                            rx + 64.0 + i as f32 * 62.0,
                            y + 504.0,
                            58.0,
                            34.0,
                            &format!("{level}"),
                            on && s.light_level == level,
                            Act::HeadLight(Some(level)),
                        );
                    }
                }
                Some(Ok(_)) => {
                    cv.text(
                        rx,
                        y + 496.0,
                        Align2::LEFT_BOTTOM,
                        &format!(
                            "{}: {}",
                            self.tr(Text::LightTitle),
                            self.tr(Text::LightNone)
                        ),
                        14.0,
                        col(t.dim),
                    );
                }
                _ => {}
            }
            cv.chip(
                rx,
                y + 552.0,
                250.0,
                40.0,
                self.tr(Text::HeadMeasure),
                false,
                Act::HeadMeasure,
            );
            match &self.head.measure {
                Some(Ok(m)) if m.ok => {
                    let l = format!(
                        "{:+.3} mm  (\u{b1} {:.3})",
                        m.height_mm().unwrap_or(0.0),
                        m.sigma_mm
                    );
                    cv.text(rx, y + 622.0, Align2::LEFT_BOTTOM, &l, 17.0, col(t.ok));
                }
                Some(Ok(m)) => {
                    cv.text(
                        rx,
                        y + 622.0,
                        Align2::LEFT_BOTTOM,
                        &m.why,
                        16.0,
                        col(t.danger),
                    );
                }
                Some(Err(e)) => {
                    let e: String = e.chars().take(60).collect();
                    cv.text(rx, y + 622.0, Align2::LEFT_BOTTOM, &e, 15.0, col(t.danger));
                }
                None => {}
            }
            // the quality check of the work
            cv.chip(
                x + 30.0,
                y + 650.0,
                280.0,
                44.0,
                self.tr(Text::InspOpen),
                false,
                Act::InspOpen,
            );
        }
        cv.chip(
            x + 330.0,
            y + 650.0,
            180.0,
            44.0,
            self.tr(Text::Save),
            false,
            Act::ConnSave,
        );
        cv.chip(
            x + 530.0,
            y + 650.0,
            180.0,
            44.0,
            self.tr(Text::Close),
            true,
            Act::CloseModal,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A node that answers two requests (the status, then a measure) and keeps what it was asked.
    fn fake_node() -> (u16, std::thread::JoinHandle<Vec<String>>) {
        use std::io::Write;
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            let mut asked = Vec::new();
            for body in [
                r#"{"node":"head-1","version":"0.0.1","camera":false,"laser":true,"rpm_sensor":true,"rpm":24000,"temp_c":40.0,"heat":"ok","protect_open":false,"uptime_s":5}"#,
                r#"{"ok":true,"why":"ok","distance_mm":100.5,"sigma_mm":0.02,"asked":7,"used":7,"rejected":0}"#,
            ] {
                let (mut s, _) = listener.accept().unwrap();
                let mut buf = [0u8; 1024];
                let n = s.read(&mut buf).unwrap();
                asked.push(
                    String::from_utf8_lossy(&buf[..n])
                        .lines()
                        .next()
                        .unwrap_or("")
                        .to_string(),
                );
                s.write_all(format!("HTTP/1.0 200 OK\r\n\r\n{body}").as_bytes())
                    .unwrap();
            }
            asked
        });
        (port, h)
    }

    #[test]
    fn the_window_asks_the_node_and_shows_what_it_says() {
        let mut a = App::new(&crate::cli::Options {
            data: Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")),
            screen: Some(crate::cli::Screen::Viewer),
            ..Default::default()
        });
        // not used: nothing is asked
        assert!(a.head_link().is_none());
        a.act(Act::HeadMode(HeadNode::Direct));
        assert!(a.head_link().is_none(), "no address yet");
        let (port, node) = fake_node();
        a.conn.head_address = format!("127.0.0.1:{port}");
        a.conn.head_token = "abcdefgh12".into();
        assert!(a.head_link().is_some());
        a.head_poll();
        for _ in 0..600 {
            a.head_tick(&egui::Context::default());
            if a.head.status.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let s = a
            .head
            .status
            .clone()
            .expect("the status")
            .expect("an answer");
        assert_eq!(
            (s.node.as_str(), s.rpm, s.protect_open),
            ("head-1", 24000.0, false)
        );
        a.act(Act::HeadMeasure);
        for _ in 0..600 {
            a.head_tick(&egui::Context::default());
            if a.head.measure.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let m = a
            .head
            .measure
            .clone()
            .expect("the measure")
            .expect("an answer");
        assert!(m.ok && (m.height_mm().unwrap() + 100.5).abs() < 1e-9);
        let asked = node.join().unwrap();
        assert_eq!(
            asked,
            vec![
                "GET /api/status HTTP/1.0".to_string(),
                "POST /api/measure HTTP/1.0".to_string()
            ]
        );
        // through the server it needs the server of the connection window and the id of the machine; the paths are the ones of the machine on the server
        a.act(Act::HeadMode(HeadNode::Server));
        assert!(a.head_link().is_none(), "no server yet");
        a.conn.server_address = "10.0.0.2".into();
        assert!(a.head_link().is_none(), "no machine yet");
        a.conn.server_machine = "bench-1".into();
        a.conn.server_token = "login".into();
        let via = a.head_link().expect("the server and the machine are set");
        assert_eq!((via.port, via.machine.as_str()), (18200, "bench-1"));
        assert_eq!(
            via.path("/api/status"),
            "/api/v1/machines/bench-1/head/status"
        );
        a.conn.server_address = "10.0.0.2:8443".into();
        assert_eq!(a.head_link().unwrap().port, 8443);
        // the choice is kept in the machine file the other programs read
        a.act(Act::HeadMode(HeadNode::Direct));
        assert!(crate::link::pairs(&a.conn)
            .iter()
            .any(|(k, v)| *k == "head_node" && v == "direct"));
    }

    #[test]
    fn an_address_has_a_port_or_the_port_of_the_web() {
        assert_eq!(
            split_address("192.168.0.60"),
            Some(("192.168.0.60".to_string(), 80))
        );
        assert_eq!(
            split_address(" head.local:8080 "),
            Some(("head.local".to_string(), 8080))
        );
        assert_eq!(split_address(""), None);
        assert_eq!(split_address("host:x"), None);
    }
}
