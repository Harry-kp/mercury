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
            // `/big` replies with a body over history's limit
            let body = if request.starts_with("GET /big") {
                serde_json::json!({ "big": "x".repeat(1_100_000) }).to_string()
            } else {
                serde_json::json!({ "echo": request }).to_string()
            };
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

/// History is read off the UI thread: step until it has arrived.
fn step_until(harness: &mut Harness<'_, MercuryApp>, done: impl Fn(&MercuryApp) -> bool) {
    for _ in 0..200 {
        harness.step();
        if done(harness.state()) {
            return;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    panic!("never happened");
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

    // `.env` is shared: its variables apply under `.env.dev`, which wins on a clash
    std::fs::write(ws.join(".env"), "SHARED=yes\nTOKEN=base\n").unwrap();
    for _ in 0..300 {
        harness.step();
        if harness.state().env_vars.contains_key("SHARED") {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(harness.state().env_name(), Some(".env.dev"));
    assert_eq!(harness.state().env_vars["SHARED"], "yes");
    assert_eq!(harness.state().env_vars["TOKEN"], "rotated");
    // the URL tooltip names the file a value is from
    assert!(harness.state().env_shared.contains("SHARED"));
    assert!(!harness.state().env_shared.contains("TOKEN"), "overridden");
    std::fs::remove_file(ws.join(".env")).unwrap();
    for _ in 0..300 {
        harness.step();
        if !harness.state().env_vars.contains_key("SHARED") {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(!harness.state().env_vars.contains_key("SHARED"));

    // A value in a JSON response goes into the environment with a right-click.
    let token = r#"{"access_token": "eyJ.abc"}"#;
    harness.state_mut().response = Some(std::sync::Arc::new(crate::http::HttpResponse {
        status: 200,
        status_text: "200 OK".into(),
        size_bytes: token.len(),
        pretty: Some(crate::http::format_json(token)),
        body: token.into(),
        content_type: "application/json".into(),
        response_type: crate::http::ResponseType::Json,
        ..crate::http::HttpResponse::failed()
    }));
    harness.state_mut().formatted_body = None;
    harness.run_steps(3);
    harness
        .get_by_label_contains("\"eyJ.abc\"")
        .click_secondary();
    harness.run_steps(2);
    harness
        .get_by_role_and_label(
            accesskit::Role::Button,
            "Save as {{access_token}} in .env.dev",
        )
        .click();
    harness.run_steps(2);
    assert_eq!(harness.state().env_vars["access_token"], "eyJ.abc");
    let dev = std::fs::read_to_string(ws.join(".env.dev")).unwrap();
    assert!(
        dev.contains("TOKEN=rotated\naccess_token=eyJ.abc\n"),
        "{dev}"
    );
    std::fs::write(
        ws.join(".env.dev"),
        dev.replace("access_token=eyJ.abc\n", ""),
    )
    .unwrap();
    harness.state_mut().response = None;
    harness.run_steps(2);

    // A body over 1 MB is shown in full, but history keeps only its status:
    // the whole history file is rewritten on every send.
    let before = harness.state().url.clone();
    harness.state_mut().url = format!("{server}/big");
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    assert!(harness.state().response.as_ref().unwrap().body.len() > 1_000_000);
    let latest = crate::storage::load_history_summaries()
        .last()
        .unwrap()
        .timestamp;
    let kept = crate::storage::load_history_entry(latest).unwrap();
    assert!(kept.response.body.is_empty());
    assert_eq!(
        kept.response.response_type,
        crate::http::ResponseType::TooLarge
    );
    harness.state_mut().url = before;

    // a binary body back from history keeps no bytes: its Save wrote the
    // "[Binary data: …]" placeholder into the file
    harness.state_mut().response = Some(std::sync::Arc::new(crate::http::HttpResponse {
        status: 200,
        status_text: "200 OK".into(),
        size_bytes: 8,
        body: "[Binary data: 8 bytes]".into(),
        content_type: "image/png".into(),
        response_type: crate::http::ResponseType::Image,
        ..crate::http::HttpResponse::failed()
    }));
    harness.run_steps(2);
    assert!(harness.query_by_label("Save response…").is_none());
    harness.get_by_label("Send again to save it");
    harness.state_mut().response = None;
    harness.run_steps(2);

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
    // a folder found by its name shows what is in it, not an empty folder
    harness.state_mut().search = "invoices".into();
    harness.run_steps(3);
    assert!(
        harness
            .query_by_role_and_label(accesskit::Role::Button, "get-invoice")
            .is_some(),
        "a folder that matches the filter must show its requests"
    );
    // nothing matching says so, instead of an empty sidebar
    harness.state_mut().search = "nothing-here".into();
    harness.run_steps(3);
    harness.get_by_label("No requests match \"nothing-here\"");
    harness.state_mut().search.clear();
    harness.run_steps(2);

    // Opening a request whose folder is already open must not rescan the
    // workspace, which reads every visible request file: on every click.
    let unseen = ws.join("users/unseen.json");
    std::fs::write(&unseen, r#"{"method":"GET","url":"u"}"#).unwrap();
    harness.state_mut().open_file(&file);
    let listed = |tree: &[crate::model::CollectionItem]| format!("{tree:?}").contains("unseen");
    assert!(
        !listed(&harness.state().tree),
        "opening a request rescanned the workspace"
    );
    std::fs::remove_file(&unseen).unwrap();
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
    // as copied from docs or a browser: `\`-continued over several lines
    // (the one-line field turned the breaks into spaces before parsing)
    harness.event(egui::Event::Paste(format!(
        "curl -X POST \\\n  {server}/x \\\n  -H 'X-A: 1' \\\n  -d 'a=1'"
    )));
    harness.run_steps(2);
    assert!(
        harness.state().current_file.is_none(),
        "the cURL must open as a new request, not take over the saved one"
    );
    assert_eq!(harness.state().method, crate::model::HttpMethod::POST);
    assert_eq!(harness.state().url, format!("{server}/x"));
    assert!(harness.state().headers_text.contains("X-A: 1"));
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

    // An untrusted certificate offers to trust that host, right on the error:
    // the host that failed, not whatever the URL bar was edited to since.
    let host = "dev.test:8443";
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
    assert!(harness.state().trusted_hosts.contains(host));
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
    // At the minimum window a row gave its URL ~5 characters next to status,
    // duration, environment and time. Narrow rows keep status and time.
    let durations = |harness: &Harness<'_, MercuryApp>| {
        // row contents aren't accessibility nodes: count the painted text
        let shapes = &harness.output().shapes;
        shapes
            .iter()
            .filter(
                |s| matches!(&s.shape, egui::Shape::Text(t) if t.galley.text().ends_with(" ms")),
            )
            .count()
    };
    assert!(
        durations(&harness) > entries.min(2),
        "a wide row shows duration: {} of {entries}",
        durations(&harness)
    );
    harness.set_size(egui::vec2(900.0, 620.0));
    harness.run_steps(3);
    assert!(
        durations(&harness) <= 1, // the response's own time may remain
        "a narrow History row still lists its duration"
    );
    harness.set_size(egui::vec2(1200.0, 800.0));
    harness.run_steps(3);
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
    step_until(&mut harness, |app| app.current_file.as_ref() == Some(&file));
    assert_eq!(
        harness.state().current_file.as_deref(),
        Some(file.as_path()),
        "a history entry must reopen the file it was sent from"
    );
    assert_eq!(harness.state().response_env.as_deref(), Some(".env.dev"));
    // ...and when that file no longer parses, the entry opens on its own
    // instead of putting its response under whatever request is open
    let original = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, "{ broken").unwrap();
    let invoice = ws.join("billing/invoices/get-invoice.json");
    harness.state_mut().open_file(&invoice);
    harness.state_mut().open_history_entry(sent.timestamp);
    step_until(&mut harness, |app| app.current_file.is_none());
    assert_eq!(harness.state().url, sent.url);
    std::fs::write(&file, original).unwrap();
    harness.state_mut().open_file(&file);
    harness.run_steps(2);

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
    // raw text wraps inside the panel, not a character or two past its edge
    small.state_mut().raw_view = true;
    small.run_steps(3);
    let row = small
        .query_all_by_label_contains("{\"echo\"")
        .next()
        .expect("the body's first row")
        .rect();
    let code_right = 900.0 - super::theme::Space::XL - super::theme::Space::LG;
    assert!(row.max.x <= code_right, "{} > {code_right}", row.max.x);
    small.state_mut().raw_view = false;
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
    step_until(&mut harness, |app| app.request_error.is_some());

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

    // Format on a body that isn't JSON says so (it did nothing at all)
    harness.state_mut().toast = None;
    harness.state_mut().body_text = "{oops".into();
    harness.state_mut().run(super::app::Action::FormatBody);
    assert!(
        harness.state().toast.as_ref().is_some_and(|t| t.is_error),
        "Format must say why it didn't"
    );
    assert_eq!(harness.state().body_text, "{oops");

    // a double-click on Send keeps the request going: its second click
    // landed on Stop, which had taken Send's place
    let silent = TcpListener::bind("127.0.0.1:0").unwrap(); // never answers
    let mut app = MercuryApp::new(&egui::Context::default());
    app.url = format!("http://{}/slow", silent.local_addr().unwrap());
    let mut dbl = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .with_step_dt(1.0 / 60.0) // clicks a frame apart, as a hand makes them
        .build_state(|ctx, app: &mut MercuryApp| app.frame(ctx), app);
    dbl.run_steps(2);
    let at = dbl
        .get_by_role_and_label(accesskit::Role::Button, "Send")
        .rect()
        .center();
    dbl.event(egui::Event::PointerMoved(at));
    for _ in 0..2 {
        for pressed in [true, false] {
            dbl.event(egui::Event::PointerButton {
                pos: at,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        dbl.step();
    }
    assert!(
        dbl.state().in_flight.is_some(),
        "a double-click on Send cancelled its own request"
    );
    drop(dbl);
    drop(silent);

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
    // a long file grew the modal past the window, title and Save off screen,
    // in Bulk edit and in the table alike: the editor scrolls instead
    let saved = e.state().env_edit_text.clone();
    e.state_mut().env_edit_text = (0..60).map(|i| format!("VAR_{i}=v{i}\n")).collect();
    for mode in ["Bulk edit", "Table"] {
        e.query_all_by_label(mode).last().unwrap().click();
        e.run_steps(5); // the modal re-centres over a few frames
        let save = e.query_all_by_label("Save").last().unwrap().rect();
        let title = e.get_by_label("Edit .env.local").rect();
        assert!(
            title.min.y >= 0.0 && save.max.y <= 800.0,
            "{mode}: {title:?} {save:?}"
        );
    }
    assert!(!e.state().env_edit_bulk, "back in the table");
    e.state_mut().env_edit_text = saved;
    e.run_steps(1);
    e.query_all_by_label("Save").last().unwrap().click();
    e.run_steps(2);
    assert!(e.state().dialog.is_none());
    e.state_mut().send_request();
    wait_for_response(&mut e);
    let echoed = e.state().response.as_ref().unwrap().body.clone();
    assert!(echoed.contains("GET /users/7 "), "{echoed}");

    // an unclosed `{{x` in the URL ran on into the headers and named a
    // variable "{{x Authorization: Bearer {{nope}}"
    e.state_mut().url = format!("{server}/{{{{x");
    e.state_mut().headers_text = "Authorization: Bearer {{nope}}".into();
    e.run_steps(2);
    assert!(e.query_by_label_contains("{{x").is_none());
    e.get_by_label_contains("{{nope}} isn't defined in .env.local");

    // two dozen environments: the menu scrolls, so New environment… (below
    // the list) stays on screen and clickable
    for i in 0..24 {
        std::fs::write(envs.join(format!(".env.e{i:02}")), "").unwrap();
    }
    for _ in 0..300 {
        e.step();
        if e.state().env_files.len() == 25 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    e.get_by_role_and_label(accesskit::Role::Button, ".env.local")
        .click();
    e.run_steps(2);
    e.query_all_by_label("New environment…")
        .next()
        .unwrap()
        .click();
    e.run_steps(2);
    assert!(matches!(e.state().dialog, Some(Dialog::NewEnv)));
    e.key_press(Key::Escape);
    e.run_steps(2);

    // the selected file deleted outside Mercury: said, not silent
    std::fs::remove_file(envs.join(".env.local")).unwrap();
    for _ in 0..300 {
        e.step();
        if e.state().env_name().is_none() {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let toast = e.state().toast.as_ref().map(|t| t.text.clone());
    assert!(
        toast
            .as_deref()
            .is_some_and(|t| t.contains(".env.local is gone")),
        "{toast:?}"
    );
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

    // An unsaved request's response belongs to it alone: replacing the form
    // with another unsaved request (⌘N, an Unsaved row, a history entry)
    // while it was in flight put the response under the new one.
    let unsaved = |url: String| crate::model::Request {
        url,
        ..Default::default()
    };
    harness
        .state_mut()
        .load_request(unsaved(format!("{server}/first")), None);
    harness.state_mut().send_request();
    harness
        .state_mut()
        .load_request(unsaved(format!("{server}/second")), None);
    wait_for_response(&mut harness);
    assert!(
        harness.state().response.is_none(),
        "/first's response was shown under /second"
    );
    assert!(harness
        .state()
        .history
        .last()
        .unwrap()
        .url
        .ends_with("/first"));
    // ...and it is still an unsaved request that was sent: Unsaved keeps it
    let recent_urls = |h: &Harness<'_, MercuryApp>| -> Vec<String> {
        h.state()
            .recent
            .iter()
            .map(|r| r.request.url.clone())
            .collect()
    };
    assert!(recent_urls(&harness).last().unwrap().ends_with("/first"));
    // a saved request sent just before switching to an unsaved one is not
    // "unsaved": it has a file
    harness.state_mut().open_file(&file);
    harness.state_mut().send_request();
    harness
        .state_mut()
        .load_request(unsaved(format!("{server}/second")), None);
    wait_for_response(&mut harness);
    assert!(
        !recent_urls(&harness)
            .iter()
            .any(|u| u.contains("{{BASE}}/users")),
        "a saved request landed in Unsaved: {:?}",
        recent_urls(&harness)
    );

    // sending an Unsaved request again moves it to the top of the list
    for path in ["/a", "/b", "/a"] {
        harness
            .state_mut()
            .load_request(unsaved(format!("{server}{path}")), None);
        harness.state_mut().send_request();
        wait_for_response(&mut harness);
    }
    let urls = recent_urls(&harness);
    assert!(urls.last().unwrap().ends_with("/a"), "{urls:?}");
    assert_eq!(urls.iter().filter(|u| u.ends_with("/a")).count(), 1);

    // saving one takes it out of Unsaved, but not a different method that
    // happens to share its URL
    harness.state_mut().method = crate::model::HttpMethod::POST;
    harness.state_mut().send_request();
    wait_for_response(&mut harness);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    harness.run_steps(2);
    harness.event(egui::Event::Text("post-a".into()));
    harness.run_steps(1);
    harness.key_press(Key::Enter);
    harness.run_steps(2);
    assert!(harness.state().current_file.is_some(), "saved");
    let left: Vec<_> = harness
        .state()
        .recent
        .iter()
        .filter(|r| r.request.url.ends_with("/a"))
        .map(|r| r.request.method)
        .collect();
    assert_eq!(left, vec![crate::model::HttpMethod::GET]);

    // a history search that matches nothing says so, not a blank panel
    harness.state_mut().show_history = true;
    harness.state_mut().history_search = "zzz-nothing".into();
    harness.run_steps(2);
    harness.get_by_label("No requests match \"zzz-nothing\"");
    harness.state_mut().history_search.clear();
    harness.state_mut().show_history = false;

    // an app-data folder Mercury can't write: the send still works, and the
    // history that won't survive a relaunch is said, not printed to stderr
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let home = dir.path().join("home");
        let mode = |m| std::fs::set_permissions(&home, std::fs::Permissions::from_mode(m));
        mode(0o555).unwrap();
        harness.state_mut().open_file(&file);
        harness.state_mut().send_request();
        wait_for_response(&mut harness);
        for _ in 0..200 {
            harness.step();
            if harness.state().toast.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        mode(0o755).unwrap();
        assert!(harness.state().response.is_some(), "the send itself works");
        let toast = harness.state().toast.as_ref().expect("an error toast");
        assert!(
            toast.is_error && toast.text.starts_with("Could not save history.json"),
            "{}",
            toast.text
        );
    }

    // quitting and relaunching reopens the request that was open, instead of
    // orphaning it into an "Untitled" unsaved request
    harness.state_mut().open_file(&file);
    harness.run_steps(2);
    eframe::App::on_exit(harness.state_mut(), None);
    let restarted = MercuryApp::new(&egui::Context::default());
    assert_eq!(restarted.current_file.as_deref(), Some(file.as_path()));
    assert!(!restarted.has_unsaved_changes());
    // history is read off the UI thread: parsing a history of big responses
    // froze the window on the first ⌘H or send (40 ms for 10 MB of it)
    let mut relaunched =
        Harness::builder().build_state(|ctx, app: &mut MercuryApp| app.frame(ctx), restarted);
    relaunched.state_mut().ensure_history_loaded();
    assert!(!relaunched.state().history_loaded, "read on the UI thread");
    step_until(&mut relaunched, |app| app.history_loaded);
    assert!(!relaunched.state().history.is_empty());
    drop(relaunched);

    // an unsaved GET comes back on Params too: restoring the last tab used
    // to override the per-request rule, so it reopened on an empty Body
    let mut scratch = MercuryApp::new(&egui::Context::default());
    scratch.load_request(Default::default(), None);
    scratch.url = "http://example.com/x".into();
    scratch.tab = Tab::Body;
    eframe::App::on_exit(&mut scratch, None);
    assert_eq!(MercuryApp::new(&egui::Context::default()).tab, Tab::Params);

    // Two-way sync with files changed outside Mercury
    let sync_dir = tempfile::TempDir::new().unwrap();
    let sws = sync_dir.path().to_path_buf();
    std::fs::create_dir(sws.join("auth")).unwrap();
    let login = sws.join("auth/login.json");
    std::fs::write(&login, r#"{"method":"POST","url":"http://a/login"}"#).unwrap();
    let mut sync = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_state(
            |ctx, app: &mut MercuryApp| app.frame(ctx),
            MercuryApp::new(&egui::Context::default()),
        );
    sync.state_mut().open_workspace(sws.clone(), None);
    sync.state_mut().open_file(&login);
    sync.run_steps(2);
    let settle = |h: &mut Harness<'_, MercuryApp>, done: &dyn Fn(&MercuryApp) -> bool| {
        // few frames, long waits: 5 s of frames would autosave over a delete
        for _ in 0..100 {
            h.step();
            if done(h.state()) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    };

    // Duplicate copies what's on screen, not the last autosave
    sync.state_mut().url.push_str("?v=2");
    sync.state_mut().duplicate(&login);
    assert!(std::fs::read_to_string(sws.join("auth/login_copy1.json"))
        .unwrap()
        .contains("?v=2"));

    // Renaming the folder of the open request keeps it open and expanded
    sync.state_mut()
        .open_dialog(Dialog::Rename(sws.join("auth")), "session");
    sync.run_steps(2);
    sync.key_press(Key::Enter);
    sync.run_steps(2);
    let login = sws.join("session/login.json");
    assert_eq!(sync.state().current_file.as_deref(), Some(login.as_path()));
    assert!(sync.state().expanded.contains(&sws.join("session")));
    assert_eq!(sync.state().opened_recently.first(), Some(&login));

    // The open file deleted on disk while it has unsaved edits: the edits
    // stay, as an unsaved request, instead of a cleared editor
    sync.state_mut().url = "http://a/edited".into();
    std::fs::remove_file(&login).unwrap();
    settle(&mut sync, &|a| a.current_file.is_none());
    assert_eq!(sync.state().current_file, None);
    assert_eq!(sync.state().url, "http://a/edited");

    // An edit in another editor reloads the request without moving the
    // editor off the tab it was on
    let x = sws.join("session/x.json");
    std::fs::write(&x, r#"{"method":"GET","url":"http://x/1"}"#).unwrap();
    sync.state_mut().open_file(&x);
    sync.state_mut().tab = Tab::Headers;
    sync.run_steps(2);
    std::fs::write(&x, r#"{"method":"GET","url":"http://x/2"}"#).unwrap();
    settle(&mut sync, &|a| a.url.ends_with("/2"));
    assert_eq!(sync.state().url, "http://x/2");
    assert_eq!(sync.state().tab, Tab::Headers);

    // A name the dialog refuses leaves it open on what was typed
    sync.state_mut()
        .open_dialog(Dialog::NewFolder(sws.clone()), "a/b");
    sync.run_steps(2);
    sync.key_press(Key::Enter);
    sync.run_steps(2);
    assert!(matches!(sync.state().dialog, Some(Dialog::NewFolder(_))));
    assert_eq!(sync.state().dialog_text, "a/b");

    // A dialog is modal: shortcuts behind it don't run. ⌘K stacked the
    // palette on it (and one Esc then closed both); ⌘H toggled history.
    sync.key_press_modifiers(Modifiers::COMMAND, Key::K);
    sync.key_press_modifiers(Modifiers::COMMAND, Key::H);
    sync.run_steps(2);
    assert!(sync.state().palette.is_none() && !sync.state().show_history);
    sync.key_press(Key::Escape);
    sync.run_steps(2);
    assert!(sync.state().dialog.is_none());

    // Creating a request from the dialog lands in the URL bar: the name
    // field had focus, and the modal still held it the frame after closing
    let url_bar = egui::Id::new("url_bar");
    let focused = |h: &Harness<'_, MercuryApp>| h.ctx.memory(|m| m.focused());
    sync.state_mut()
        .open_dialog(Dialog::NewRequest(sws.clone(), false), "");
    sync.run_steps(2);
    sync.event(egui::Event::Text("fresh".into()));
    sync.key_press(Key::Enter);
    sync.run_steps(3);
    assert!(sync.state().current_file.is_some());
    assert_eq!(focused(&sync), Some(url_bar));
    // ...and so does New request from the palette; ⌘K again closes it
    sync.key_press_modifiers(Modifiers::COMMAND, Key::K);
    sync.run_steps(2);
    sync.key_press_modifiers(Modifiers::COMMAND, Key::K);
    sync.run_steps(2);
    assert!(sync.state().palette.is_none(), "⌘K toggles the palette");
    sync.key_press_modifiers(Modifiers::COMMAND, Key::K);
    sync.run_steps(2);
    sync.event(egui::Event::Text("new request".into()));
    sync.key_press(Key::Enter);
    sync.run_steps(3);
    assert!(sync.state().current_file.is_none());
    assert_eq!(focused(&sync), Some(url_bar));

    // #150: the URL bar keeps focus through ⌘⏎ and the response
    sync.state_mut().url = format!("{server}/x");
    sync.key_press_modifiers(Modifiers::COMMAND, Key::Enter);
    wait_for_response(&mut sync);
    assert!(sync.state().response.is_some());
    assert_eq!(focused(&sync), Some(url_bar));

    // Esc that closes a menu is spent: it used to clear the filter too
    // (or cancel the request in flight)
    sync.state_mut().search = "x".into();
    sync.run_steps(2);
    sync.get_by_role_and_label(accesskit::Role::Button, "More")
        .click();
    sync.run_steps(2);
    sync.key_press(Key::Escape);
    sync.run_steps(2);
    assert_eq!(sync.state().search, "x");

    // the body's layout is kept between frames: finding egui's cached copy
    // meant re-highlighting and hashing it, 35 ms a frame for a 1 MB body
    sync.state_mut().tab = Tab::Body;
    sync.state_mut().body_text = r#"{"a": 1}"#.into();
    sync.run_steps(2);
    let galley = |h: &Harness<'_, MercuryApp>| {
        let layout = h.state().body_layout.as_ref().expect("body laid out");
        layout.galley.clone()
    };
    // a stand-in egui never made: laid out again, it would be replaced
    let marker = sync
        .ctx
        .fonts_mut(|f| f.layout_no_wrap("x".into(), egui::FontId::default(), egui::Color32::RED));
    sync.state_mut().body_layout.as_mut().unwrap().galley = marker.clone();
    sync.run_steps(2);
    assert!(std::sync::Arc::ptr_eq(&marker, &galley(&sync)));
    sync.state_mut().body_text.push(' ');
    sync.run_steps(1);
    assert!(!std::sync::Arc::ptr_eq(&marker, &galley(&sync)));
    // and not kept while another tab is showing
    sync.state_mut().tab = Tab::Headers;
    sync.run_steps(1);
    assert!(sync.state().body_layout.is_none());
}
