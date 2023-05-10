use leptos::*;
use leptos_meta::Body;
use leptos_router::ActionForm;

#[server(ToggleDarkMode, "/api")]
pub async fn toggle_dark_mode(cx: Scope, prefers_dark: bool) -> Result<bool, ServerFnError> {
    use axum::http::{header::SET_COOKIE, HeaderMap, HeaderValue};
    use leptos_axum::{ResponseOptions, ResponseParts};

    let response =
        use_context::<ResponseOptions>(cx).expect("to have leptos_axum::ResponseOptions provided");
    let mut response_parts = ResponseParts::default();
    let mut headers = HeaderMap::new();
    headers.insert(
        SET_COOKIE,
        HeaderValue::from_str(&format!("darkmode={prefers_dark}; Path=/"))
            .expect("to create header value"),
    );
    response_parts.headers = headers;

    response.overwrite(response_parts);
    Ok(prefers_dark)
}

#[cfg(not(feature = "ssr"))]
fn initial_prefers_dark(_cx: Scope) -> Option<bool> {
    use wasm_bindgen::JsCast;

    let doc = document().unchecked_into::<web_sys::HtmlDocument>();
    let query = window()
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .and_then(|ql| ql.map(|ql| ql.matches()));
    let cookie = doc.cookie().unwrap_or_default();
    if cookie.contains("darkmode") {
        Some(cookie.contains("darkmode=true"))
    } else {
        query
    }
}

#[cfg(feature = "ssr")]
fn initial_prefers_dark(cx: Scope) -> Option<bool> {
    use axum_extra::extract::cookie::CookieJar;
    use_context::<leptos_axum::RequestParts>(cx).and_then(|req| {
        let cookies = CookieJar::from_headers(&req.headers);
        cookies.get("darkmode").and_then(|v| match v.value() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        })
    })
}

#[component]
pub fn DarkModeToggle(cx: Scope) -> impl IntoView {
    let initial = initial_prefers_dark(cx);
    let toggle_dark_mode_action = create_server_action::<ToggleDarkMode>(cx);
    // input is `Some(value)` when pending, and `None` if not pending
    let input = toggle_dark_mode_action.input();
    // value contains most recently-returned value
    let value = toggle_dark_mode_action.value();
    let prefers_dark = create_memo(cx, move |_| {
        match (input(), value()) {
            // if there's some current input, use that optimistically
            (Some(submission), _) => Some(submission.prefers_dark),
            // otherwise, if there was a previous value confirmed by server, use that
            (_, Some(Ok(value))) => Some(value),
            // otherwise, use the initial value
            _ => initial,
        }
    });
    let prefers_dark = move || {
        if cfg!(feature = "ssr") {
            initial
        } else {
            prefers_dark()
        }
    };

    view! { cx,
        <Body class=move || match prefers_dark() {
            Some(true) => "dark".to_string(),
            Some(false) => "light".to_string(),
            _ => "bg-white".to_string(),
        }/>
        <script>r#"
            if(window.matchMedia('(prefers-color-scheme: dark)').matches) {
            	document.body.classList.add('dark');
            } else {
            	document.body.classList.remove('dark');
            }"#
        </script>
        <ActionForm action=toggle_dark_mode_action>
            <input
                type="hidden"
                name="prefers_dark"
                value=move || (!(prefers_dark().unwrap_or(false))).to_string()
            />
            <button
                type="submit"
                value=move || { if prefers_dark().unwrap_or(false) { "dark" } else { "light" } }
            >
                <img
                    class=" h-6 w-6 block"
                    src=move || {
                        if prefers_dark().unwrap_or(false) { "/images/sun.svg" } else { "/images/moon.svg" }
                    }
                    alt="Toggle Dark Mode"
                />
            </button>
        </ActionForm>
    }
}
