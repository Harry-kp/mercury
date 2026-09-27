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
            // `/status/404` replies 404, so failing statuses can be exercised
            let code = request
                .split_whitespace()
                .nth(1)
                .and_then(|p| p.strip_prefix("/status/"))
                .and_then(|c| c.split('?').next())
                .unwrap_or("200");
            let status = format!("{code} Test");
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nSet-Cookie: s=1; Path=/\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
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
    let server = echo_server();
    std::fs::write(
        ws.join(".env.dev"),
        format!("BASE={server}\nTOKEN=secret-token\n"),
    )
    .unwrap();
    // a request two folders deep that the test never expands: the sidebar
    // filter has to reach it
    std::fs::create_dir_all(ws.join("billing/invoices")).unwrap();
    std::fs::write(
        ws.join("billing/invoices/get-invoice.json"),
        r#"{"method":"GET","url":"{{BASE}}/invoices/42"}"#,
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

    // a new env file that sorts first must not shift the selection, and
    // edits to the selected file take effect (file watcher -> refresh)
    std::fs::write(ws.join(".env.a-first"), "TOKEN=wrong\n").unwrap();
    let dev = std::fs::read_to_string(ws.join(".env.dev")).unwrap();
    std::fs::write(ws.join(".env.dev"), dev.replace("secret-token", "rotated")).unwrap();
    for _ in 0..300 {
        harness.step();
        if harness.state().env_files.len() == 2
            && harness.state().env_vars.get("TOKEN").map(String::as_str) == Some("rotated")
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(harness.state().env_name(), Some(".env.dev"));
    assert_eq!(harness.state().env_vars["TOKEN"], "rotated");

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

    // the filter reaches requests inside folders that were never expanded
    harness.state_mut().search = "invoice".into();
    harness.run_steps(3);
    assert!(
        harness.query_by_label("get-invoice").is_some(),
        "the sidebar filter must search the whole workspace, not just open folders"
    );
    harness.state_mut().search.clear();
    harness.run_steps(2);

    // history records the request that was sent, even if the form changes
    // while it is in flight
    let sent_url = harness.state().url.clone();
    harness.state_mut().send_request();
    harness.state_mut().url = "https://edited-after-send.example.com".into();
    wait_for_response(&mut harness);
    let recorded = harness.state().history.last().expect("history entry");
    assert_eq!(
        recorded.url, sent_url,
        "history must pair the response with the request that produced it"
    );
    harness.state_mut().url = sent_url;
    harness.run_steps(2);

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

    // a 4xx is not a success: the notification must not read as one
    harness.state_mut().url = format!("{server}/status/404");
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    assert!(
        harness.state().toast.as_ref().is_some_and(|t| t.is_error),
        "a failing status must not be announced as a success"
    );

    // a response that lands after the user opened another request belongs to
    // neither of them on screen, so it goes to history only
    harness.state_mut().open_file(&file);
    harness.run_steps(2);
    harness.state_mut().send_request();
    harness.run_steps(1);
    harness
        .state_mut()
        .open_file(&ws.join("billing/invoices/get-invoice.json"));
    wait_for_response(&mut harness);
    assert!(
        harness.state().response.is_none(),
        "a response was attributed to a request that did not make it"
    );
    assert!(
        harness
            .state()
            .history
            .last()
            .is_some_and(|h| h.url.contains("users")),
        "the sent request was lost from history"
    );

    // quitting and relaunching reopens the request that was open, instead of
    // orphaning it into an "Untitled" unsaved request
    harness.state_mut().open_file(&file);
    harness.run_steps(2);
    eframe::App::on_exit(harness.state_mut(), None);
    let restarted = MercuryApp::new(&egui::Context::default());
    assert_eq!(restarted.current_file.as_deref(), Some(file.as_path()));
    assert!(!restarted.has_unsaved_changes());
}
