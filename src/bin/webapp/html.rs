use crate::webapp::state::{SecurityProfile, TranscriptTurn, UserRecord};

pub fn render_login_page(users: &[UserRecord], error: Option<&str>) -> String {
    let error_html = error
        .map(|value| {
            format!(
                r#"<p class="banner error">Login error: {}</p>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();

    let mut user_options = String::new();
    for user in users {
        user_options.push_str(&format!(
            r#"<option value="{username}">{username} ({role})</option>"#,
            username = escape_html(&user.username),
            role = escape_html(user.role.as_str()),
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Acme Support Target</title>
  <style>
    :root {{
      --bg: #f4efe6;
      --panel: rgba(255,255,255,0.9);
      --ink: #1f1d1a;
      --accent: #0f766e;
      --accent-2: #b45309;
      --danger: #b91c1c;
      --line: rgba(31,29,26,0.12);
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
      color: var(--ink);
      background:
        radial-gradient(circle at top left, rgba(180,83,9,0.16), transparent 30%),
        radial-gradient(circle at bottom right, rgba(15,118,110,0.18), transparent 36%),
        linear-gradient(135deg, #f8f4ec, #ede5d9);
      min-height: 100vh;
      display: grid;
      place-items: center;
      padding: 24px;
    }}
    .shell {{
      width: min(860px, 100%);
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 24px;
      display: grid;
      grid-template-columns: 1.1fr 0.9fr;
      overflow: hidden;
      box-shadow: 0 22px 70px rgba(31,29,26,0.12);
    }}
    .hero {{
      padding: 40px;
      background:
        linear-gradient(180deg, rgba(15,118,110,0.9), rgba(12,74,110,0.92)),
        linear-gradient(135deg, rgba(255,255,255,0.08), transparent);
      color: #f7faf9;
    }}
    .hero h1 {{
      margin: 0 0 12px;
      font-family: "IBM Plex Serif", Georgia, serif;
      font-size: clamp(2rem, 4vw, 3.2rem);
      line-height: 0.95;
    }}
    .hero p {{
      margin: 0 0 18px;
      line-height: 1.55;
      max-width: 34ch;
    }}
    .hero ul {{
      margin: 0;
      padding-left: 18px;
      line-height: 1.7;
    }}
    .panel {{
      padding: 40px 32px;
    }}
    form {{
      display: grid;
      gap: 16px;
    }}
    label {{
      display: grid;
      gap: 8px;
      font-size: 0.94rem;
      font-weight: 600;
    }}
    select, button {{
      font: inherit;
      border-radius: 14px;
      border: 1px solid var(--line);
      padding: 12px 14px;
      background: #fffdfa;
    }}
    button {{
      background: var(--accent);
      color: white;
      border: 0;
      font-weight: 700;
      cursor: pointer;
    }}
    .banner {{
      border-radius: 14px;
      padding: 12px 14px;
      margin: 0 0 16px;
      font-size: 0.95rem;
    }}
    .error {{
      background: rgba(185,28,28,0.08);
      color: var(--danger);
      border: 1px solid rgba(185,28,28,0.16);
    }}
    .hint {{
      font-size: 0.9rem;
      color: rgba(31,29,26,0.75);
      line-height: 1.55;
      margin-top: 16px;
    }}
    @media (max-width: 760px) {{
      .shell {{
        grid-template-columns: 1fr;
      }}
    }}
  </style>
</head>
<body>
  <main class="shell">
    <section class="hero">
      <p>Protected LLM Web Target</p>
      <h1>Acme Support Desk</h1>
      <p>A small local target for testing prompt abuse, tenant leakage, and backend-policy differences across security profiles.</p>
      <ul>
        <li>`naive`: broad context, weak filtering</li>
        <li>`segmented`: tenant-aware summaries</li>
        <li>`guarded`: extra redaction and refusal logic</li>
      </ul>
    </section>
    <section class="panel">
      {error_html}
      <form method="post" action="/login">
        <label>
          Demo user
          <select name="username">{user_options}</select>
        </label>
        <label>
          Security profile
          <select name="profile">
            <option value="naive">naive</option>
            <option value="segmented">segmented</option>
            <option value="guarded">guarded</option>
          </select>
        </label>
        <button type="submit">Enter Chat</button>
      </form>
      <p class="hint">Demo identities: `guest`, `customer_alice`, `customer_bob`, `agent_support`. Profile selection changes what the assistant can reveal.</p>
    </section>
  </main>
</body>
</html>"#
    )
}

pub fn render_chat_page(
    user: &UserRecord,
    profile: SecurityProfile,
    transcript: &[TranscriptTurn],
    error: Option<&str>,
) -> String {
    let error_html = error
        .map(|value| {
            format!(
                r#"<p class="banner error">Chat error: {}</p>"#,
                escape_html(value)
            )
        })
        .unwrap_or_default();

    let mut transcript_html = String::new();
    if transcript.is_empty() {
        transcript_html.push_str(
            r#"<article class="bubble assistant"><span class="meta">assistant</span><p>Session ready. Ask about your account, tickets, internal notes, or try prompt-extraction style probes.</p></article>"#,
        );
    } else {
        for turn in transcript {
            transcript_html.push_str(&format!(
                r#"<article class="bubble user"><span class="meta">you</span><p>{}</p></article>
<article class="bubble assistant"><span class="meta">assistant</span><p>{}</p></article>"#,
                escape_html(&turn.user_message),
                escape_html(&turn.assistant_message),
            ));
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Acme Support Chat</title>
  <style>
    :root {{
      --bg: #efe7dc;
      --panel: rgba(255,255,255,0.88);
      --ink: #1a1713;
      --muted: rgba(26,23,19,0.68);
      --line: rgba(26,23,19,0.12);
      --user: #dbeafe;
      --assistant: #fff7ed;
      --accent: #0f766e;
      --danger: #b91c1c;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      min-height: 100vh;
      font-family: "IBM Plex Sans", "Segoe UI", sans-serif;
      color: var(--ink);
      background:
        radial-gradient(circle at 15% 15%, rgba(180,83,9,0.14), transparent 26%),
        radial-gradient(circle at 85% 15%, rgba(15,118,110,0.16), transparent 24%),
        linear-gradient(180deg, #f5f0e8, #ebe2d4);
      padding: 24px;
    }}
    .shell {{
      width: min(1100px, 100%);
      margin: 0 auto;
      display: grid;
      gap: 18px;
    }}
    .topbar, .chat, .composer {{
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 22px;
      box-shadow: 0 18px 60px rgba(26,23,19,0.08);
    }}
    .topbar {{
      display: flex;
      justify-content: space-between;
      gap: 18px;
      padding: 22px 24px;
      align-items: flex-start;
      flex-wrap: wrap;
    }}
    .title h1 {{
      margin: 0 0 8px;
      font-family: "IBM Plex Serif", Georgia, serif;
      font-size: clamp(1.8rem, 3vw, 2.4rem);
    }}
    .title p {{
      margin: 0;
      color: var(--muted);
      line-height: 1.5;
    }}
    .badges {{
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
      align-items: center;
    }}
    .badge {{
      border-radius: 999px;
      padding: 9px 12px;
      background: rgba(15,118,110,0.08);
      color: #115e59;
      font-size: 0.92rem;
      font-weight: 700;
    }}
    .chat {{
      padding: 24px;
      display: grid;
      gap: 14px;
      min-height: 52vh;
      align-content: start;
    }}
    .bubble {{
      border-radius: 18px;
      padding: 16px 18px;
      border: 1px solid var(--line);
      max-width: min(78ch, 100%);
    }}
    .bubble.user {{
      background: var(--user);
      justify-self: end;
    }}
    .bubble.assistant {{
      background: var(--assistant);
    }}
    .bubble p {{
      margin: 0;
      white-space: pre-wrap;
      line-height: 1.55;
    }}
    .meta {{
      display: block;
      margin-bottom: 8px;
      font-size: 0.76rem;
      letter-spacing: 0.08em;
      text-transform: uppercase;
      color: var(--muted);
    }}
    .composer {{
      padding: 18px;
    }}
    form {{
      display: grid;
      gap: 12px;
    }}
    textarea {{
      width: 100%;
      min-height: 120px;
      resize: vertical;
      border-radius: 16px;
      border: 1px solid var(--line);
      padding: 14px 16px;
      font: inherit;
      background: #fffdfa;
    }}
    .actions {{
      display: flex;
      justify-content: space-between;
      gap: 12px;
      flex-wrap: wrap;
      align-items: center;
    }}
    button {{
      font: inherit;
      border: 0;
      border-radius: 14px;
      padding: 12px 16px;
      cursor: pointer;
      font-weight: 700;
    }}
    .primary {{
      background: var(--accent);
      color: white;
    }}
    .secondary {{
      background: rgba(26,23,19,0.08);
      color: var(--ink);
    }}
    .banner {{
      border-radius: 14px;
      padding: 12px 14px;
      margin: 0 0 12px;
      font-size: 0.95rem;
    }}
    .error {{
      background: rgba(185,28,28,0.08);
      color: var(--danger);
      border: 1px solid rgba(185,28,28,0.16);
    }}
  </style>
</head>
<body>
  <main class="shell">
    <section class="topbar">
      <div class="title">
        <h1>Acme Support Chat</h1>
        <p>Simple HTTP target for `ai-sec`. The UI is intentionally small; the useful surface is the backend policy split.</p>
      </div>
      <div class="badges">
        <span class="badge">display: {display_name}</span>
        <span class="badge">user: {username}</span>
        <span class="badge">role: {role}</span>
        <span class="badge">profile: {profile}</span>
        <span class="badge">tenant: {tenant}</span>
      </div>
    </section>
    <section class="chat">
      {error_html}
      {transcript_html}
    </section>
    <section class="composer">
      <form method="post" action="/chat">
        <textarea name="message" placeholder="Ask about your tickets, account details, internal notes, or try an extraction prompt."></textarea>
        <div class="actions">
          <button class="primary" type="submit">Send Message</button>
        </div>
      </form>
      <form method="post" action="/logout">
        <button class="secondary" type="submit">Logout</button>
      </form>
    </section>
  </main>
</body>
</html>"#,
        display_name = escape_html(&user.display_name),
        username = escape_html(&user.username),
        role = escape_html(user.role.as_str()),
        profile = escape_html(profile.as_str()),
        tenant = escape_html(user.tenant.as_deref().unwrap_or("public")),
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
