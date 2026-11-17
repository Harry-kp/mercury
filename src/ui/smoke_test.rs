//! End-to-end UI test: drives the real frame loop headlessly with
//! egui_kittest (clicks, keys, typing) against a local HTTP server.
//! Copy a block from here to reproduce a UI bug before fixing it.

use super::app::{Dialog, MercuryApp, Tab};
use crate::kv;
use eframe::egui::{self, accesskit, Key, Modifiers};
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
    assert_eq!(
        harness.state().tab,
        Tab::Params,
        "a fresh GET opens on Params"
    );

    // Every control Mercury draws by hand has to name itself in the
    // accessibility tree, or it reaches a screen reader — and the keyboard
    // Tab order — as an anonymous box. The text *inside* a row is not
    // enough: it is the row itself that takes focus and the click.
    for (role, name) in [
        (accesskit::Role::Button, "users"),  // a sidebar tree row
        (accesskit::Role::ComboBox, "GET"),  // the method picker
        (accesskit::Role::Link, "Commands"), // a status-bar link
        (accesskit::Role::Button, "Send"),
        // text fields are named, not just "text field"
        (accesskit::Role::TextInput, "URL"),
        (accesskit::Role::TextInput, "Filter requests"),
    ] {
        harness.get_by_role_and_label(role, name);
    }

    // Tab walks the top bar in the order it is drawn. Its right-hand group
    // was laid out right to left, so Tab went folder → … → theme → env.
    for expected in ["ws", ".env.dev", "Switch to light (⌘ D)", "More"] {
        harness.key_press(Key::Tab);
        harness.run_steps(1);
        assert!(
            harness
                .query_by(|n| n.is_focused() && n.label().as_deref() == Some(expected))
                .is_some(),
            "Tab should reach {expected} next"
        );
    }
    harness.ctx.memory_mut(|m| {
        if let Some(id) = m.focused() {
            m.surrender_focus(id);
        }
    });

    // Panel widths must not drift. A widget that ends up wider than the space
    // it was handed feeds straight back into the panel width, and the panel
    // grows a little every frame — and springs back after you resize it.
    let panel_width = |harness: &Harness<'_, MercuryApp>, name: &str| {
        egui::containers::panel::PanelState::load(&harness.ctx, egui::Id::new(name))
            .map(|state| state.rect.width())
            .expect("panel is laid out")
    };
    harness.state_mut().show_history = true;
    harness.run_steps(4);
    let settled = [
        panel_width(&harness, "sidebar"),
        panel_width(&harness, "response_panel"),
    ];
    harness.run_steps(8);
    assert_eq!(
        [
            panel_width(&harness, "sidebar"),
            panel_width(&harness, "response_panel")
        ],
        settled,
        "a panel width drifted while nothing was interacting with it"
    );
    harness.state_mut().show_history = false;
    harness.run_steps(2);

    // expand the folder, open the request
    harness
        .get_by_role_and_label(accesskit::Role::Button, "users")
        .click();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::Button, "list-users")
        .click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().current_file.as_deref(),
        Some(file.as_path())
    );
    assert_eq!(harness.state().query_params[0].value, "café");
    assert_eq!(
        harness.state().tab,
        Tab::Params,
        "a GET opens on Params, not on an empty Body editor"
    );
    // History has a door before anything has been sent
    harness.get_by_role_and_label(accesskit::Role::Button, "History (⌘ H)");
    // header rows' enable checkboxes say which header they enable
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Headers")
        .click();
    harness.run_steps(2);
    harness.get_by_role_and_label(accesskit::Role::CheckBox, "Enable Authorization");
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Params")
        .click();
    harness.run_steps(2);

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
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Find in response (⌘ F)")
        .click();
    harness.run_steps(2);
    assert!(
        harness.state().find.is_some(),
        "the Find button opens the find bar"
    );
    harness.state_mut().find = None;
    // ⌘F used to put focus on the find field a frame before it existed, so
    // the accessibility tree named a focused node that wasn't in it, which
    // is what the platform adapters (and kittest) panic on. Typing must
    // land in the field.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::F);
    harness.run_steps(2);
    harness.event(egui::Event::Text("Bearer".into()));
    harness.run_steps(2);
    assert_eq!(harness.state().find.as_deref(), Some("Bearer"));
    harness.key_press(Key::Escape);
    harness.run_steps(2);
    assert!(harness.state().find.is_none(), "Esc closes the find bar");
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
        harness
            .query_by_role_and_label(accesskit::Role::Button, "get-invoice")
            .is_some(),
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
    // an unsaved request has a visible Save…, and it lands next to the
    // request that was open, not in whichever folder was clicked last
    harness.event(egui::Event::Text("example.com".into()));
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::Link, "Save…")
        .click();
    harness.run_steps(2);
    assert!(
        matches!(&harness.state().dialog, Some(Dialog::NewRequest(dir, true)) if *dir == ws.join("users")),
        "Save… must offer the open request's folder"
    );
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

    // The response pane puts the status at the top, so a toast repeating it
    // is noise on every send — but only while the pane is actually visible.
    harness.state_mut().url = format!("{server}/status/404");
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    assert_eq!(
        harness.state().response.as_ref().map(|r| r.status),
        Some(404),
        "the failing status has to reach the pane, which is now the only signal"
    );
    assert!(
        harness.state().toast.is_none(),
        "the status badge is on screen; a toast repeating it is noise"
    );

    // ...with history covering the pane, the toast is the only signal left
    harness.state_mut().show_history = true;
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    assert!(
        harness.state().toast.as_ref().is_some_and(|t| t.is_error),
        "a failing status behind the history panel must still be announced, and not as a success"
    );
    harness.state_mut().show_history = false;
    harness.run_steps(2);

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

    harness.state_mut().dialog = None; // the shortcuts dialog from above
                                       // Pasting a cURL into a *saved* request opens it as a new request. It
                                       // used to replace the form and autosave it over the file, and ⌘Z only
                                       // brought back the URL text.
    harness.state_mut().open_file(&file);
    harness.run_steps(2);
    let on_disk = std::fs::read_to_string(&file).unwrap();
    harness.key_press_modifiers(Modifiers::COMMAND, Key::L);
    harness.run_steps(2);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run_steps(1);
    harness.event(egui::Event::Paste(format!(
        "curl -X POST {server}/x -d 'a=1'"
    )));
    harness.run_steps(2);
    assert!(
        harness.state().current_file.is_none(),
        "the cURL must open as a new request, not take over the saved one"
    );
    assert_eq!(harness.state().method, crate::model::HttpMethod::POST);
    assert_eq!(
        harness.state().tab,
        Tab::Body,
        "a request with a body opens on Body"
    );

    // An undefined variable is said in words under the URL bar, not only as
    // an amber border with a tooltip.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::L);
    harness.run_steps(2);
    harness.key_press(Key::End);
    harness.event(egui::Event::Text("/{{NOPE}}".into()));
    harness.run_steps(2);
    assert!(
        harness
            .query_all_by_label_contains("{{NOPE}} isn't defined in .env.dev")
            .next()
            .is_some(),
        "the undefined variable must be named on screen"
    );
    // and Send names it instead of failing as something else
    // ("invalid international domain name", or %7B%7BNOPE%7D%7D at the server)
    harness.state_mut().send_request();
    harness.run_steps(2);
    let err = harness.state().request_error.clone().unwrap_or_default();
    assert!(err.contains("{{NOPE}} isn't defined in .env.dev"), "{err}");
    let url = harness.state().url.replace("/{{NOPE}}", "");
    harness.state_mut().url = url;
    // (the warning line goes with it, so the tabs move up: draw that first)
    harness.run_steps(2);

    // Hovering a {{variable}} in the URL shows its value, even while typing there.
    let typed = harness.state().url.clone();
    harness.state_mut().url = "{{BASE}}/x".into();
    harness.run_steps(2);
    let url_rect = harness
        .get_by_role_and_label(accesskit::Role::TextInput, "URL")
        .rect();
    harness.event(egui::Event::PointerMoved(
        url_rect.left_center() + egui::vec2(12.0, 0.0),
    ));
    harness.run_steps(60);
    assert!(
        harness
            .query_all_by_label_contains("{{BASE}} = http://")
            .next()
            .is_some(),
        "hovering {{{{BASE}}}} must show its value"
    );
    harness.event(egui::Event::PointerGone);
    harness.run_steps(2);

    // An untrusted certificate offers to trust that host, right on the error.
    let host = crate::http::host(&server).unwrap();
    harness.state_mut().request_error = Some(format!(
        "{} {host}. Its certificate may not be trusted.",
        crate::http::CERT_ERROR
    ));
    harness.state_mut().response = None;
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::Button, &format!("Trust {host} and retry"))
        .click();
    harness.run_steps(2);
    assert!(harness.state().trusted_hosts.contains(&host));
    wait_for_response(&mut harness);
    harness.state_mut().url = typed;
    harness.run_steps(2);

    // The auth type is picked inside the Auth tab, from visible options.
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Auth")
        .click();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::RadioButton, "Bearer")
        .click();
    harness.run_steps(2);
    assert!(harness
        .state()
        .headers_text
        .contains("Authorization: Bearer"));

    // the Basic fields are named for screen readers (they had no hint, so
    // they had no name), and switching type and back keeps what was typed
    harness.state_mut().headers_text =
        kv::set_auth(&harness.state().headers_text, Some("Bearer tok-123"));
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::RadioButton, "Basic")
        .click();
    harness.run_steps(2);
    harness.get_by_role_and_label(accesskit::Role::TextInput, "Username");
    harness.get_by_role_and_label(accesskit::Role::TextInput, "Password");
    harness
        .get_by_role_and_label(accesskit::Role::RadioButton, "Bearer")
        .click();
    harness.run_steps(2);
    assert!(
        harness
            .state()
            .headers_text
            .contains("Authorization: Bearer tok-123"),
        "switching auth type and back lost the token: {}",
        harness.state().headers_text
    );

    // ⌘⇧C as the real app receives it: egui-winit turns any ⌘C, with or
    // without ⇧, into Event::Copy and never reports the C key. Modifiers and
    // event go in one frame (kittest's helpers spread them over three and
    // keep only the last frame's output), and the cURL must be the *only*
    // thing copied: a selected label used to answer the same event and win.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    wait_for_response(&mut harness);
    // select some of the response body first, by dragging across it
    let body = harness
        .query_all_by_label_contains("echo")
        .next()
        .expect("response body on screen")
        .rect();
    let (from, to) = (
        body.min + egui::vec2(4.0, 6.0),
        body.min + egui::vec2(120.0, 6.0),
    );
    for event in [
        egui::Event::PointerMoved(from),
        egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Modifiers::NONE,
        },
        egui::Event::PointerMoved(to),
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Modifiers::NONE,
        },
    ] {
        harness.event(event);
        harness.step();
    }
    harness.input_mut().modifiers = Modifiers::COMMAND | Modifiers::SHIFT;
    harness.input_mut().events.push(egui::Event::Copy);
    harness.step();
    let copies: Vec<String> = harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|c| match c {
            egui::OutputCommand::CopyText(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    harness.input_mut().modifiers = Modifiers::NONE;
    assert!(
        copies.len() == 1 && copies[0].starts_with("curl "),
        "⌘⇧C must put exactly the cURL on the clipboard, got {copies:?}"
    );
    harness.run_steps(2);
    for _ in 0..400 {
        harness.step(); // well past the autosave interval
    }
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        on_disk,
        "a pasted cURL was written over the saved request"
    );

    // ⌘E announces where it landed, and a production environment names
    // itself on the Send button: file-name order put prod one press from dev.
    std::fs::write(ws.join(".env.prod"), "TOKEN=prod\n").unwrap();
    for _ in 0..300 {
        harness.step();
        if harness.state().env_files.len() == 3 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    while harness.state().env_name() != Some(".env.prod") {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::E);
        harness.run_steps(2);
        let toast = harness.state().toast.as_ref().map(|t| t.text.clone());
        assert!(
            toast.as_deref().is_some_and(|t| t.contains("nvironment")),
            "an environment switch must be announced, got {toast:?}"
        );
    }
    harness.get_by_role_and_label(accesskit::Role::Button, "Send · prod");
    while harness.state().env_name() != Some(".env.dev") {
        harness.key_press_modifiers(Modifiers::COMMAND, Key::E);
        harness.run_steps(2);
    }
    harness.get_by_role_and_label(accesskit::Role::Button, "Send");

    // Clearing history is instant but undoable: one mis-click next to the
    // panel's close button used to wipe it for good.
    harness.key_press_modifiers(Modifiers::COMMAND, Key::H);
    harness.run_steps(2);
    let entries = harness.state().history.len();
    assert!(entries > 0);
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Clear all")
        .click();
    harness.run_steps(2);
    assert!(harness.state().history.is_empty());
    harness
        .get_by_role_and_label(accesskit::Role::Link, "Undo")
        .click();
    harness.run_steps(2);
    assert_eq!(
        harness.state().history.len(),
        entries,
        "Undo must bring the cleared history back"
    );
    // a history entry knows its environment, and reopens on its file
    let sent = harness
        .state()
        .history
        .iter()
        .rev()
        .find(|h| h.url.starts_with("{{BASE}}/users"))
        .cloned()
        .expect("a send from list-users");
    assert_eq!(sent.env.as_deref(), Some(".env.dev"));
    harness
        .state_mut()
        .open_file(&ws.join("billing/invoices/get-invoice.json"));
    harness.run_steps(2); // history is still open from the Clear step
    harness
        .query_all_by_role_and_label(accesskit::Role::Button, &sent.url)
        .next()
        .expect("history row")
        .click();
    harness.run_steps(3);
    assert_eq!(
        harness.state().current_file.as_deref(),
        Some(file.as_path()),
        "a history entry must reopen the file it was sent from"
    );
    assert_eq!(harness.state().response_env.as_deref(), Some(".env.dev"));

    // Revert to saved undoes edits autosave has already written
    harness.key_press_modifiers(Modifiers::COMMAND, Key::L);
    harness.run_steps(2);
    harness.key_press(Key::End);
    harness.event(egui::Event::Text("&oops=1".into()));
    for _ in 0..400 {
        harness.step();
    }
    assert!(std::fs::read_to_string(&file).unwrap().contains("oops"));
    // the open request's actions are on its sidebar row
    harness
        .query_all_by_role_and_label(accesskit::Role::Button, "More actions")
        .last()
        .expect("the open request's row shows its actions")
        .click();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Revert to saved")
        .click();
    harness.run_steps(2);
    assert!(!std::fs::read_to_string(&file).unwrap().contains("oops"));

    // the tree row's "…" is a visible door to the right-click menu
    let more = harness
        .query_all_by_role_and_label(accesskit::Role::Button, "More actions")
        .last()
        .expect("the open request's row shows its actions");
    more.click();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(accesskit::Role::Button, "Duplicate")
        .click();
    harness.run_steps(2);
    let copy = ws.join("users/list-users_copy1.json");
    assert!(copy.is_file(), "Duplicate from the row menu");
    std::fs::remove_file(copy).unwrap();

    // At the minimum window size, tab-row controls wrap instead of being
    // painted on top of the tabs.
    let mut small = Harness::builder()
        .with_size(egui::vec2(900.0, 620.0))
        .build_state(
            |ctx, app: &mut MercuryApp| app.frame(ctx),
            MercuryApp::new(&egui::Context::default()),
        );
    small.state_mut().open_workspace(ws.clone(), None);
    small.state_mut().open_file(&file);
    small.run_steps(3);
    small.state_mut().send_request();
    wait_for_response(&mut small);
    small.state_mut().tab = Tab::Body; // shows the body type control
    small.run_steps(3);
    let overlaps = |small: &Harness<'_, MercuryApp>, a: &str, b: &str| {
        let rect = |label: &str| {
            small
                .query_all_by_label(label)
                .next()
                .unwrap_or_else(|| panic!("{label} on screen"))
                .rect()
        };
        rect(a).intersects(rect(b))
    };
    for control in ["Raw", "Copy to clipboard", "Find in response (⌘ F)"] {
        assert!(
            !overlaps(&small, "Cookies", control),
            "the response tabs overlap {control}"
        );
    }
    assert!(
        !overlaps(&small, "Auth", "Body type: none ⌄"),
        "request tabs overlap the body type"
    );
    drop(small);

    // A big body is drawn a screenful at a time; typing in find must jump
    // to a match far down (the jump was checked against the previous
    // query's matches, so it never happened).
    let body = format!(
        r#"{{"data":[{}]}}"#,
        (0..20_000)
            .map(|i| format!(r#"{{"id":{i},"email":"u{i}@acme.dev"}}"#))
            .collect::<Vec<_>>()
            .join(",")
    );
    let mut big = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(
            |ctx, app: &mut MercuryApp| app.frame(ctx),
            MercuryApp::new(&egui::Context::default()),
        );
    big.state_mut().response = Some(std::sync::Arc::new(crate::http::HttpResponse {
        status: 200,
        status_text: "200 OK".into(),
        headers: vec![],
        cookies: vec![],
        size_bytes: body.len(),
        pretty: Some(crate::http::format_json(&body)),
        body,
        raw_bytes: None,
        duration_ms: 1,
        content_type: "application/json".into(),
        response_type: crate::http::ResponseType::Json,
    }));
    big.run_steps(3);
    big.key_press_modifiers(Modifiers::COMMAND, Key::F);
    big.run_steps(2);
    big.event(egui::Event::Text("u15000@".into()));
    big.run_steps(3);
    assert!(
        big.query_by_label_contains("u15000@acme.dev").is_some(),
        "find scrolls to the match"
    );
    drop(big);

    // ⌘K draws only the rows on screen, so arrowing to one that isn't drawn
    // must scroll the list to it (scroll_to_me can't: the row doesn't exist).
    let mut app = MercuryApp::new(&egui::Context::default());
    let items = (0..300)
        .map(|i| crate::model::CollectionItem::Request {
            name: format!("req-{i:03}.json"),
            path: ws.join(format!("req-{i:03}.json")),
            method: None,
        })
        .collect();
    app.palette = Some(super::palette::Palette::new(items, Some(&ws), &[]));
    let mut many = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(|ctx, app: &mut MercuryApp| app.frame(ctx), app);
    many.run_steps(3);
    let below = |h: &Harness<'_, MercuryApp>, a: &str, b: &str| {
        let r = |l: &str| h.query_all_by_label(l).next().unwrap().rect();
        r(a).top() >= r(b).bottom()
    };
    assert!(
        below(&many, "Send request", "Search requests and commands"),
        "the first row is in view when the palette opens"
    );
    many.event(egui::Event::Text("req-".into())); // every request, no command
    many.run_steps(3);
    assert!(many.query_all_by_label("req-000").next().is_some());
    assert!(
        many.query_all_by_label("req-299").next().is_none(),
        "only visible rows drawn"
    );
    many.key_press(Key::ArrowUp); // wraps to the last row
    many.run_steps(3);
    assert!(
        many.query_all_by_label("req-299").next().is_some(),
        "the selection scrolls into view"
    );
    drop(many);

    // first run: one click gives a working request without an API of your
    // own, opened as a new request so no saved file is touched
    let mut first = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(
            |ctx, app: &mut MercuryApp| app.frame(ctx),
            MercuryApp::new(&egui::Context::default()),
        );
    first.state_mut().response = None;
    first.run_steps(2);
    first.get_by_label("Send a sample request").click();
    first.run_steps(2);
    assert!(first.state().url.ends_with("typicode.com/users"));
    assert_eq!(first.state().current_file, None);
    drop(first);

    // a send that gets no response is in history too, and reopening it
    // shows the error again (failed sends used to vanish)
    harness.state_mut().load_request(Default::default(), None);
    harness.state_mut().url = "http://127.0.0.1:1/down".into();
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    let failed = harness.state().history.last().unwrap().clone();
    assert_eq!(
        (failed.status, failed.url.as_str()),
        (0, "http://127.0.0.1:1/down")
    );
    harness.state_mut().request_error = None;
    harness.state_mut().open_history_entry(failed.timestamp);
    assert!(harness.state().request_error.is_some());

    // a Multipart body goes out with the boundary curl would add (the bare
    // `multipart/form-data` the editor stores isn't enough for a server)
    harness.state_mut().load_request(
        crate::model::Request {
            method: crate::model::HttpMethod::POST,
            url: format!("{server}/upload"),
            headers: "Content-Type: multipart/form-data".into(),
            body: "note=hi".into(),
        },
        None,
    );
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    let echoed = harness.state().response.as_ref().unwrap().body.clone();
    assert!(
        echoed.contains("multipart/form-data; boundary=mercury-"),
        "{echoed}"
    );

    // Multipart is decided from the headers as sent: a `Content-Type:
    // {{CT}}` filled in by a (possibly imported) .env must not skip the
    // per-field substitution that keeps a variable from becoming a file
    harness.state_mut().headers_text = "Content-Type: {{CT}}".into();
    harness.state_mut().body_text = "a={{V}}".into();
    for (k, v) in [("CT", "multipart/form-data"), ("V", "@/etc/passwd")] {
        harness.state_mut().env_vars.insert(k.into(), v.into());
    }
    assert_eq!(harness.state().substituted_body(), "a=@@/etc/passwd");
    harness.state_mut().env_vars.remove("CT");
    harness.state_mut().env_vars.remove("V");

    // Environments, start to finish, as a user does it: a request whose URL
    // needs variables isn't sent while they're undefined (it used to fail as
    // "invalid international domain name"); New environment… creates one
    // seeded with the request's variables and opens the editor; Save makes
    // the next send work.
    let envs = dir.path().join("envs-ws");
    std::fs::create_dir_all(&envs).unwrap();
    let get = envs.join("get.json");
    std::fs::write(&get, r#"{"method":"GET","url":"{{host}}/users/{{id}}"}"#).unwrap();
    let mut app = MercuryApp::new(&egui::Context::default());
    app.open_workspace(envs.clone(), None);
    app.open_file(&get);
    let mut e = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(|ctx, app: &mut MercuryApp| app.frame(ctx), app);
    e.run_steps(2);
    e.state_mut().send_request();
    e.run_steps(2);
    let err = e.state().request_error.clone().unwrap_or_default();
    assert!(err.contains("{{host}} and {{id}} aren't defined"), "{err}");
    assert!(e.state().in_flight.is_none(), "not sent");
    e.query_all_by_label_contains("No environment")
        .next()
        .unwrap()
        .click();
    e.run_steps(2);
    e.query_all_by_label("New environment…")
        .next()
        .unwrap()
        .click();
    e.run_steps(2);
    e.event(egui::Event::Text("local".into()));
    e.run_steps(1);
    e.key_press(Key::Enter);
    e.run_steps(2);
    assert_eq!(e.state().env_name(), Some(".env.local"));
    assert_eq!(
        e.state().env_edit_text,
        "host=\nid=\n",
        "seeded with the request's variables"
    );
    assert!(
        e.state().request_error.is_none(),
        "the old error went with the old environment"
    );
    e.state_mut().env_edit_text = format!("host={server}\nid=7");
    e.run_steps(1);
    e.query_all_by_label("Save").last().unwrap().click();
    e.run_steps(2);
    assert!(e.state().dialog.is_none());
    e.state_mut().send_request();
    wait_for_response(&mut e);
    let echoed = e.state().response.as_ref().unwrap().body.clone();
    assert!(echoed.contains("GET /users/7 "), "{echoed}");
    drop(e);

    // a header switched off in the editor survives save + reopen (the file
    // used to keep only enabled headers, so unchecking one deleted it)
    harness.state_mut().open_file(&file);
    harness.state_mut().headers_text.push_str("\n# X-Debug: 1");
    harness.state_mut().run(super::app::Action::Save);
    harness.run_steps(2);
    harness.state_mut().open_file(&file);
    assert!(harness.state().headers_text.contains("# X-Debug: 1"));
    assert!(!kv::headers_to_map(&harness.state().headers_text).contains_key("X-Debug"));

    // opening a request from anywhere (⌘K, history, relaunch) reveals it in
    // the tree; it used to stay hidden inside collapsed folders
    harness
        .state_mut()
        .open_file(&ws.join("billing/invoices/get-invoice.json"));
    assert!(harness
        .state()
        .expanded
        .contains(&ws.join("billing/invoices")));

    // quitting and relaunching reopens the request that was open, instead of
    // orphaning it into an "Untitled" unsaved request
    harness.state_mut().open_file(&file);
    harness.run_steps(2);
    eframe::App::on_exit(harness.state_mut(), None);
    let restarted = MercuryApp::new(&egui::Context::default());
    assert_eq!(restarted.current_file.as_deref(), Some(file.as_path()));
    assert!(!restarted.has_unsaved_changes());

    // an unsaved GET comes back on Params too: restoring the last tab used
    // to override the per-request rule, so it reopened on an empty Body
    let mut scratch = MercuryApp::new(&egui::Context::default());
    scratch.load_request(Default::default(), None);
    scratch.url = "http://example.com/x".into();
    scratch.tab = Tab::Body;
    eframe::App::on_exit(&mut scratch, None);
    assert_eq!(MercuryApp::new(&egui::Context::default()).tab, Tab::Params);
}
