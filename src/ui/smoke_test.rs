//! End-to-end UI test: drives the real frame loop headlessly with
//! egui_kittest (clicks, keys, typing) against a local HTTP server.
//! Copy a block from here to reproduce a UI bug before fixing it.

use super::app::{Dialog, MercuryApp};
use eframe::egui::{self, Key, Modifiers};
use egui_kittest::{kittest::Queryable, Harness};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

/// Replies 200 JSON whose body is the raw request it received.
fn echo_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let body = serde_json::json!({ "echo": request }).to_string();
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nSet-Cookie: s=1; Path=/\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
        }
    });
    format!("http://{addr}")
}

fn wait_for_response(harness: &mut Harness<'_, MercuryApp>) {
    for _ in 0..200 {
        harness.step();
        if harness.state().in_flight.is_none() {
            harness.run_steps(2);
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("request did not finish");
}

#[test]
fn open_send_edit_and_save() {
    let dir = tempfile::TempDir::new().unwrap();
    // SAFETY: the only test in the crate that touches env vars / storage on disk
    unsafe { std::env::set_var("MERCURY_HOME", dir.path().join("home")) };
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(ws.join("users")).unwrap();
    std::fs::write(
        ws.join(".env.dev"),
        format!("BASE={}\nTOKEN=secret-token\n", echo_server()),
    )
    .unwrap();
    let file = ws.join("users/list-users.json");
    std::fs::write(
        &file,
        r#"{"method":"GET","url":"{{BASE}}/users?q=caf%C3%A9","headers":{"Authorization":"Bearer {{TOKEN}}"}}"#,
    )
    .unwrap();

    let ctx = egui::Context::default();
    let mut app = MercuryApp::new(&ctx);
    app.open_workspace(ws.clone(), None);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(|ctx, app: &mut MercuryApp| app.frame(ctx), app);
    harness.run_steps(2);
    assert_eq!(harness.state().env_name(), Some(".env.dev"));

    // expand the folder, open the request
    harness.get_by_label("users").click();
    harness.run_steps(2);
    harness.get_by_label("list-users").click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().current_file.as_deref(),
        Some(file.as_path())
    );
    assert_eq!(harness.state().query_params[0].value, "café");

    // ⌘Enter sends; env vars are substituted into URL and headers
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    wait_for_response(&mut harness);
    let response = harness.state().response.clone().expect("response shown");
    assert_eq!(response.status, 200);
    assert!(
        response.body.contains("GET /users?q=caf%C3%A9"),
        "{}",
        response.body
    );
    assert!(
        response.body.contains("authorization: Bearer secret-token"),
        "{}",
        response.body
    );
    assert_eq!(response.cookies, vec!["s=1; Path=/"]);
    assert_eq!(harness.state().history.len(), 1);
    // saved requests don't go to Recent
    assert!(harness.state().recent.is_empty());

    // typing "?" in the URL bar must not open the shortcuts dialog
    harness.key_press_modifiers(Modifiers::COMMAND, Key::L); // focus URL bar
    harness.run_steps(2);
    harness.event(egui::Event::Key {
        key: Key::End,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    harness.event(egui::Event::Text("&x?".into()));
    harness.run_steps(2);
    assert!(harness.state().dialog.is_none());
    assert!(harness.state().url.ends_with("&x?"));

    // ⌘S writes the file (autosave may already have, after 5s of frame time)
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    harness.run_steps(2);
    assert!(!harness.state().has_unsaved_changes());
    assert!(std::fs::read_to_string(&file).unwrap().contains("&x?"));

    // ⌘N starts an unsaved request; "?" outside a text field opens help
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    harness.run_steps(2);
    assert!(harness.state().current_file.is_none());
    harness.state_mut().dialog = None;
    harness.remove_cursor();
    harness.event(egui::Event::Key {
        key: Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    harness.run_steps(1);
    harness.key_press_modifiers(Modifiers::SHIFT, Key::Questionmark);
    harness.run_steps(2);
    assert!(matches!(harness.state().dialog, Some(Dialog::Shortcuts)));
}
