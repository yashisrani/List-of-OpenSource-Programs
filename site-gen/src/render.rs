//! HTML rendering.
//!
//! Every page is emitted whole, with the full program list in the initial
//! markup: the site must work without JavaScript and be crawlable, since the
//! whole point is that someone googling "GSoC 2026 stipend" can land here.
//! Filtering is layered on afterwards by app.js.

use std::fmt::Write as _;

use crate::model::{Program, Site};

pub const SITE_NAME: &str = "Open Source Programs";
pub const SITE_URL: &str = "https://list-of-opensource-programs.vercel.app";
pub const REPO_URL: &str = "https://github.com/yashisrani/List-of-OpenSource-Programs";

/// HTML-escape text content.
pub fn esc(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&#39;"),
            _ => o.push(c),
        }
    }
    o
}

/// Escape text, then turn `backtick spans` into styled code labels.
///
/// The YAML carries a little inline markdown for things like GitHub issue
/// labels, where naming the label is the whole point. Escaping runs first, so
/// the content inside the backticks can never inject markup; only the fence
/// characters themselves become tags. An unmatched trailing backtick is left as
/// a literal, which is what a contributor typing prose would expect.
pub fn esc_code(s: &str) -> String {
    let escaped = esc(s);
    let mut out = String::with_capacity(escaped.len());
    let mut rest = escaped.as_str();
    loop {
        let Some(open) = rest.find('`') else {
            out.push_str(rest);
            return out;
        };
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            out.push_str(rest);
            return out;
        };
        out.push_str(&rest[..open]);
        out.push_str("<code class=\"code-label\">");
        out.push_str(&after[..close]);
        out.push_str("</code>");
        rest = &after[close + 1..];
    }
}

pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August",
    "September", "October", "November", "December",
];

/// Inline SVG sprite. Icons are drawn once here rather than pulled from an icon
/// font so the page ships zero extra requests. These are traced from Lucide's
/// MIT-licensed set.
mod icon {
    pub const SEARCH: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>"#;
    pub const EXTERNAL: &str = r#"<svg class="ext" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M7 17 17 7M9 7h8v8"/></svg>"#;
    pub const ALERT: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><path d="M12 9v4M12 17h.01"/><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/></svg>"#;
    pub const SUN: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M6.3 17.7l-1.4 1.4M19.1 4.9l-1.4 1.4"/></svg>"#;
    pub const GITHUB: &str = r#"<svg viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 .5C5.7.5.5 5.7.5 12c0 5.1 3.3 9.4 7.9 10.9.6.1.8-.2.8-.6v-2c-3.2.7-3.9-1.5-3.9-1.5-.5-1.3-1.3-1.7-1.3-1.7-1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.7 1.3 3.4 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.8 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.4 11.4 0 0 1 6 0C17.6 4.7 18.6 5 18.6 5c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.5-2.7 5.5-5.3 5.8.4.4.8 1.1.8 2.2v3.3c0 .4.2.7.8.6 4.6-1.5 7.9-5.8 7.9-10.9C23.5 5.7 18.3.5 12 .5z"/></svg>"#;
}

pub struct Page<'a> {
    pub title: &'a str,
    pub description: &'a str,
    pub path: &'a str,
    pub nav: &'a str,
    pub body: String,
    /// Extra JSON-LD injected into <head>, already serialized.
    pub json_ld: Option<String>,
}

pub fn shell(p: &Page) -> String {
    let canonical = if p.path == "/" {
        SITE_URL.to_string()
    } else {
        format!("{SITE_URL}{}", p.path)
    };
    let full_title = if p.path == "/" {
        format!("{SITE_NAME} 2026 - Paid Open Source Internships and Fellowships")
    } else {
        format!("{} - {SITE_NAME}", p.title)
    };

    let mut h = String::with_capacity(p.body.len() + 4096);
    h.push_str("<!doctype html>\n<html lang=\"en\">\n<head>\n");
    h.push_str("<meta charset=\"utf-8\">\n");
    h.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(h, "<title>{}</title>", esc(&full_title));
    let _ = writeln!(h, "<meta name=\"description\" content=\"{}\">", esc(p.description));
    let _ = writeln!(h, "<link rel=\"canonical\" href=\"{}\">", esc(&canonical));
    h.push_str("<meta name=\"robots\" content=\"index,follow\">\n");

    // Open Graph / Twitter.
    let _ = writeln!(h, "<meta property=\"og:type\" content=\"website\">");
    let _ = writeln!(h, "<meta property=\"og:site_name\" content=\"{SITE_NAME}\">");
    let _ = writeln!(h, "<meta property=\"og:title\" content=\"{}\">", esc(&full_title));
    let _ = writeln!(h, "<meta property=\"og:description\" content=\"{}\">", esc(p.description));
    let _ = writeln!(h, "<meta property=\"og:url\" content=\"{}\">", esc(&canonical));
    let _ = writeln!(h, "<meta name=\"twitter:card\" content=\"summary_large_image\">");

    h.push_str("<link rel=\"icon\" href=\"/favicon.svg\" type=\"image/svg+xml\">\n");
    h.push_str("<link rel=\"stylesheet\" href=\"/assets/app.css\">\n");

    // Set the theme before first paint so a dark-mode user never sees a white
    // flash. Kept inline and tiny for exactly that reason.
    h.push_str(
        "<script>try{var t=localStorage.getItem('theme');if(t)document.documentElement.setAttribute('data-theme',t);}catch(e){}</script>\n",
    );

    if let Some(ld) = &p.json_ld {
        let _ = writeln!(h, "<script type=\"application/ld+json\">{ld}</script>");
    }

    h.push_str("</head>\n<body>\n");
    h.push_str("<a class=\"skip-link\" href=\"#main\">Skip to content</a>\n");

    // Header.
    h.push_str("<header class=\"site-head\"><div class=\"wrap\">\n");
    let _ = write!(h, "<a class=\"brand\" href=\"/\">{SITE_NAME}</a>\n");
    h.push_str("<nav class=\"nav\" aria-label=\"Main\">\n");
    for (href, label, key) in [
        ("/", "Programs", "programs"),
        ("/timeline/", "Timeline", "timeline"),
        ("/start-here/", "Start here", "start"),
        ("/resources/", "Resources", "resources"),
    ] {
        let cur = if p.nav == key { " aria-current=\"page\"" } else { "" };
        let _ = writeln!(h, "<a href=\"{href}\"{cur}>{label}</a>");
    }
    h.push_str("</nav>\n");
    let _ = write!(
        h,
        "<button class=\"icon-btn theme-btn\" type=\"button\" aria-label=\"Switch theme\">{}</button>\n",
        icon::SUN
    );
    let _ = write!(
        h,
        "<a class=\"icon-btn\" href=\"{REPO_URL}\" aria-label=\"View this project on GitHub\" rel=\"noopener\">{}</a>\n",
        icon::GITHUB
    );
    h.push_str("</div></header>\n");

    let _ = write!(h, "<main id=\"main\">\n{}\n</main>\n", p.body);

    // Footer.
    h.push_str("<footer class=\"site-foot\"><div class=\"wrap\">\n");
    let _ = write!(
        h,
        "<div>Community-maintained. Verify every date on the official site before applying.</div>\n"
    );
    h.push_str("<div class=\"foot-right\">\n");
    let _ = writeln!(h, "<a href=\"{REPO_URL}\" rel=\"noopener\">Source</a>");
    let _ = writeln!(
        h,
        "<a href=\"{REPO_URL}/blob/main/Readme.md\" rel=\"noopener\">Readme</a>"
    );
    let _ = writeln!(
        h,
        "<a href=\"{REPO_URL}/issues/new\" rel=\"noopener\">Report an outdated entry</a>"
    );
    h.push_str("</div>\n</div></footer>\n");

    h.push_str("<script src=\"/assets/app.js\" defer></script>\n");
    h.push_str("</body>\n</html>\n");
    h
}

/// One program row.
fn program_row(p: &Program) -> String {
    let mut haystack = format!("{} {} ", p.name, p.org);
    if let Some(s) = &p.short_name {
        haystack.push_str(s);
        haystack.push(' ');
    }
    if let Some(e) = &p.eligibility {
        haystack.push_str(e);
        haystack.push(' ');
    }
    if let Some(t) = &p.timeline {
        haystack.push_str(t);
    }

    let mut h = String::new();
    let _ = write!(
        h,
        "<li class=\"row\" data-row data-tags=\"{}\" data-search-text=\"{}\">\n",
        esc(&p.tags.join(" ")),
        esc(&haystack)
    );

    // Main column.
    h.push_str("<div class=\"row-main\">\n");
    let _ = write!(
        h,
        "<h3 class=\"row-title\"><a href=\"{}\" rel=\"noopener\">{}{}</a></h3>\n",
        esc(&p.url),
        esc(p.display_name()),
        icon::EXTERNAL
    );
    let _ = write!(h, "<div class=\"row-org\">{}</div>\n", esc(&p.org));
    if let Some(e) = &p.eligibility {
        let _ = write!(h, "<p class=\"row-elig\">{}</p>\n", esc(e));
    }
    // Only tags that actually distinguish this row get rendered. `paid`/`unpaid`
    // would just restate the stipend column, and `global` sits on nearly every
    // entry, so both are filter-only. All tags stay in `data-tags` above so the
    // chips keep working.
    let shown: Vec<&String> = p
        .tags
        .iter()
        .filter(|t| matches!(t.as_str(), "beginner-friendly" | "underrepresented" | "students"))
        .collect();
    if !shown.is_empty() {
        h.push_str("<div class=\"tags\">\n");
        for t in shown {
            let _ = write!(h, "<span class=\"tag t-{}\">{}</span>", slugify(t), esc(t));
        }
        h.push_str("\n</div>\n");
    }
    if let Some(c) = &p.caveat {
        let _ = write!(
            h,
            "<p class=\"caveat\">{}<span>{}</span></p>\n",
            icon::ALERT,
            esc(c)
        );
    }
    h.push_str("</div>\n");

    // Timeline column.
    h.push_str("<div class=\"row-when\">\n");
    h.push_str("<span class=\"when-label\">Timeline</span>\n");
    match &p.timeline {
        Some(t) => {
            let _ = write!(h, "<span class=\"when-text\">{}</span>\n", esc(t));
        }
        None => h.push_str("<span class=\"when-text\">Not announced</span>\n"),
    }
    h.push_str("</div>\n");

    // Money column.
    h.push_str("<div class=\"row-pay\">\n");
    h.push_str("<span class=\"pay-label\">Stipend</span>\n");
    if let Some(r) = &p.rewards {
        let _ = write!(h, "<div class=\"amount is-vague\">{}</div>\n", esc(r));
    } else if let Some(s) = &p.stipend {
        let vague = if s.has_amount() { "" } else { " is-vague" };
        let _ = write!(h, "<div class=\"amount{vague}\">{}</div>\n", esc(&s.display()));
        if let Some(n) = s.sub_note() {
            let _ = write!(h, "<div class=\"pay-note\">{}</div>\n", esc(n));
        }
    } else {
        h.push_str("<div class=\"amount is-vague\">Not stated</div>\n");
    }
    h.push_str("</div>\n");

    h.push_str("</li>\n");
    h
}

pub fn home(site: &Site) -> Page<'_> {
    let mut b = String::new();

    let paid = site.programs.iter().filter(|p| p.is_paid()).count();
    let total = site.programs.len() + site.competitions.len();

    b.push_str("<div class=\"page-head\"><div class=\"wrap\">\n");
    b.push_str("<h1>Open source programs and internships for 2026</h1>\n");
    let _ = write!(
        b,
        "<p class=\"lede\">Every active mentorship, fellowship and competition worth applying to, with stipends and deadlines in one place. <strong>{paid} of {total} are paid.</strong></p>\n"
    );
    b.push_str("</div></div>\n");

    // Filter bar.
    b.push_str("<div class=\"filters\"><div class=\"wrap\">\n");
    let _ = write!(
        b,
        "<div class=\"search\">{}<label class=\"sr-only\" for=\"q\">Search programs</label><input id=\"q\" type=\"search\" placeholder=\"Search programs, organizations...\" data-search autocomplete=\"off\"></div>\n",
        icon::SEARCH
    );
    b.push_str("<div class=\"chips\" role=\"group\" aria-label=\"Filter by tag\">\n");
    for (tag, label) in [
        ("paid", "Paid"),
        ("beginner-friendly", "Beginner friendly"),
        ("students", "Students"),
        ("global", "Open worldwide"),
    ] {
        let _ = write!(
            b,
            "<button class=\"chip\" type=\"button\" data-tag=\"{tag}\" aria-pressed=\"false\">{label}</button>\n"
        );
    }
    b.push_str("</div>\n");
    let _ = write!(b, "<span class=\"result-count\" data-count>{total} programs</span>\n");
    b.push_str("</div></div>\n");
    b.push_str("<p class=\"sr-only\" role=\"status\" aria-live=\"polite\" data-live></p>\n");

    // Programs, highest stipend first so the most consequential options lead.
    let mut ordered: Vec<&Program> = site.programs.iter().collect();
    ordered.sort_by(|a, b| {
        b.stipend_sort()
            .cmp(&a.stipend_sort())
            .then_with(|| a.name.cmp(&b.name))
    });

    b.push_str("<section class=\"section\" style=\"border-top:0;padding-top:26px\"><div class=\"wrap\">\n");
    b.push_str("<ul class=\"list\" data-list>\n");
    for p in &ordered {
        b.push_str(&program_row(p));
    }
    for p in &site.competitions {
        b.push_str(&program_row(p));
    }
    b.push_str("</ul>\n");

    b.push_str("<div class=\"empty\" data-empty hidden>\n");
    b.push_str("<h2>No programs match</h2>\n");
    b.push_str("<p>Try a different search term or clear the filters.</p>\n");
    b.push_str("<button type=\"button\" data-reset>Clear filters</button>\n");
    b.push_str("</div>\n");
    b.push_str("</div></section>\n");

    Page {
        title: "Programs",
        description: "A curated list of active open source programs, internships and fellowships for 2026, with stipends, timelines and eligibility for GSoC, Outreachy, LFX, MLH and more.",
        path: "/",
        nav: "programs",
        body: b,
        json_ld: Some(home_json_ld(site)),
    }
}

/// ItemList structured data so search engines can surface individual programs.
fn home_json_ld(site: &Site) -> String {
    let items: Vec<serde_json::Value> = site
        .programs
        .iter()
        .chain(site.competitions.iter())
        .enumerate()
        .map(|(i, p)| {
            serde_json::json!({
                "@type": "ListItem",
                "position": i + 1,
                "name": p.name,
                "url": p.url,
            })
        })
        .collect();
    serde_json::json!({
        "@context": "https://schema.org",
        "@type": "ItemList",
        "name": "Open source programs and internships 2026",
        "itemListElement": items,
    })
    .to_string()
}

pub fn timeline(site: &Site) -> Page<'_> {
    let mut b = String::new();
    let year = site.timeline.year;

    b.push_str("<div class=\"page-head\"><div class=\"wrap\">\n");
    let _ = write!(b, "<h1>{year} timeline</h1>\n");
    b.push_str("<p class=\"lede\">Application windows and contribution seasons for the programs on this list, in order. Dates marked approximate have not been confirmed by the organizers.</p>\n");
    b.push_str("</div></div>\n");

    b.push_str("<section class=\"section\" style=\"border-top:0\"><div class=\"wrap\">\n");
    b.push_str("<div class=\"months\">\n");

    // Look up a program's URL by name or short_name so events can link out.
    let find_url = |needle: &str| -> Option<&str> {
        site.programs
            .iter()
            .chain(site.competitions.iter())
            .find(|p| {
                p.name.eq_ignore_ascii_case(needle)
                    || p.short_name.as_deref().map(|s| s.eq_ignore_ascii_case(needle)) == Some(true)
            })
            .map(|p| p.url.as_str())
    };

    for m in 1..=12u32 {
        let events: Vec<_> = site
            .timeline
            .events
            .iter()
            .filter(|e| e.month == m)
            .collect();
        if events.is_empty() {
            continue;
        }
        let name = MONTHS[(m - 1) as usize];
        let _ = write!(b, "<div class=\"month\" id=\"{}\">\n", slugify(name));
        let _ = write!(b, "<h3>{name} {year}</h3>\n");
        b.push_str("<ul class=\"month-events\">\n");
        for e in events {
            let approx = if e.approx { " is-approx" } else { "" };
            let _ = write!(b, "<li class=\"event{approx}\">\n");
            let _ = write!(b, "<span class=\"date\">{}</span>\n", esc(&e.date));
            b.push_str("<span class=\"what\">");
            match e.program.as_deref().and_then(find_url) {
                Some(url) => {
                    let _ = write!(
                        b,
                        "<a href=\"{}\" rel=\"noopener\">{}</a>",
                        esc(url),
                        esc(&e.event)
                    );
                }
                None => b.push_str(&esc(&e.event)),
            }
            if let Some(t) = &e.time {
                let _ = write!(b, "<span class=\"meta\">{}</span>", esc(t));
            }
            if e.approx {
                b.push_str("<span class=\"meta\">approx</span>");
            }
            b.push_str("</span>\n</li>\n");
        }
        b.push_str("</ul>\n</div>\n");
    }

    b.push_str("</div>\n</div></section>\n");

    Page {
        title: "2026 Timeline",
        description: "Month-by-month 2026 timeline of open source program deadlines: GSoC organization and contributor applications, Outreachy cohorts, Season of KDE, Hacktoberfest and more.",
        path: "/timeline/",
        nav: "timeline",
        body: b,
        json_ld: None,
    }
}

pub fn start_here(site: &Site) -> Page<'_> {
    let g = &site.guide;
    let mut b = String::new();

    b.push_str("<div class=\"page-head\"><div class=\"wrap\">\n");
    b.push_str("<h1>Start here</h1>\n");
    b.push_str("<p class=\"lede\">New to open source? These programs have the shortest path from zero to a merged pull request, and none of them need you to be an expert first.</p>\n");
    b.push_str("</div></div>\n");

    // Beginner picks.
    b.push_str("<section class=\"section\" style=\"border-top:0\"><div class=\"wrap\">\n");
    b.push_str("<h2>Best programs for beginners</h2>\n");
    b.push_str("<p class=\"section-lede\">Ranked by how little ceremony stands between you and your first contribution.</p>\n");
    b.push_str("<ul class=\"stack\">\n");
    for pick in &g.best_for_beginners {
        let url = site
            .programs
            .iter()
            .chain(site.competitions.iter())
            .find(|p| p.name.eq_ignore_ascii_case(&pick.program))
            .map(|p| p.url.as_str());
        b.push_str("<li>\n<div class=\"item-name\">");
        match url {
            Some(u) => {
                let _ = write!(b, "<a href=\"{}\" rel=\"noopener\">{}</a>", esc(u), esc(&pick.program));
            }
            None => b.push_str(&esc(&pick.program)),
        }
        b.push_str("</div>\n");
        let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc_code(&pick.why));
        let _ = write!(
            b,
            "<p class=\"item-desc\"><strong>Getting started:</strong> {}</p>\n",
            esc_code(&pick.getting_started)
        );
        b.push_str("</li>\n");
    }
    b.push_str("</ul>\n</div></section>\n");

    // Languages.
    b.push_str("<section class=\"section\"><div class=\"wrap\">\n");
    b.push_str("<h2>Where to find your first issue</h2>\n");
    b.push_str("<p class=\"section-lede\">Repositories with an active habit of labelling work for newcomers, and the labels each community actually uses.</p>\n");
    b.push_str("<div class=\"cols-2\">\n<div>\n");
    let half = g.languages.len().div_ceil(2);
    for (i, lang) in g.languages.iter().enumerate() {
        if i == half {
            b.push_str("</div>\n<div>\n");
        }
        b.push_str("<div class=\"lang-block\">\n");
        let _ = write!(b, "<h3>{}</h3>\n", esc(&lang.name));
        if !lang.repos.is_empty() {
            b.push_str("<div class=\"repo-links\">\n");
            for r in &lang.repos {
                let _ = write!(
                    b,
                    "<a href=\"{}\" rel=\"noopener\">{}</a>",
                    esc(&r.url),
                    esc(&r.name)
                );
            }
            b.push_str("\n</div>\n");
        }
        if let Some(n) = &lang.note {
            let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc_code(n));
        }
        if !lang.labels.is_empty() {
            b.push_str("<div class=\"label-list\">\n");
            for l in &lang.labels {
                let _ = write!(b, "<span class=\"code-label\">{}</span>", esc(l));
            }
            b.push_str("\n</div>\n");
        }
        b.push_str("</div>\n");
    }
    b.push_str("</div>\n</div>\n</div></section>\n");

    // Non-code + learning path, side by side.
    b.push_str("<section class=\"section\"><div class=\"wrap\">\n");
    b.push_str("<div class=\"cols-2\">\n");

    b.push_str("<div>\n<h2>You do not have to write code</h2>\n");
    b.push_str("<p class=\"section-lede\">Every one of these counts as a real contribution.</p>\n");
    b.push_str("<ul class=\"stack\">\n");
    for nc in &g.non_code {
        b.push_str("<li>\n");
        let _ = write!(b, "<div class=\"item-name\">{}</div>\n", esc(&nc.kind));
        let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc_code(&nc.description));
        if !nc.links.is_empty() {
            b.push_str("<div class=\"repo-links\" style=\"margin-top:7px\">\n");
            for l in &nc.links {
                let _ = write!(
                    b,
                    "<a href=\"{}\" rel=\"noopener\">{}</a>",
                    esc(&l.url),
                    esc(&l.name)
                );
            }
            b.push_str("\n</div>\n");
        }
        b.push_str("</li>\n");
    }
    b.push_str("</ul>\n</div>\n");

    b.push_str("<div>\n<h2>An eight week path</h2>\n");
    b.push_str("<p class=\"section-lede\">A realistic ramp from never having used Git to submitting your first application.</p>\n");
    b.push_str("<ol class=\"steps\">\n");
    for s in &g.learning_path {
        b.push_str("<li>\n");
        let _ = write!(b, "<div class=\"step-when\">{}</div>\n", esc(&s.weeks));
        let _ = write!(b, "<div class=\"step-goal\">{}</div>\n", esc_code(&s.goal));
        b.push_str("</li>\n");
    }
    b.push_str("</ol>\n</div>\n");

    b.push_str("</div>\n</div></section>\n");

    // Labels.
    b.push_str("<section class=\"section\"><div class=\"wrap\">\n");
    b.push_str("<h2>Labels worth searching for</h2>\n");
    b.push_str("<p class=\"section-lede\">Search any of these on GitHub together with a language you know.</p>\n");
    b.push_str("<ul class=\"stack\">\n");
    for l in &g.labels {
        b.push_str("<li>\n");
        let _ = write!(
            b,
            "<div class=\"item-name\"><span class=\"code-label\">{}</span></div>\n",
            esc(&l.label)
        );
        let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc_code(&l.meaning));
        b.push_str("</li>\n");
    }
    b.push_str("</ul>\n</div></section>\n");

    Page {
        title: "Start here",
        description: "A guide for first-time open source contributors: the most beginner-friendly programs, where to find your first issue by language, non-code ways to contribute, and an eight week learning path.",
        path: "/start-here/",
        nav: "start",
        body: b,
        json_ld: None,
    }
}

pub fn resources(site: &Site) -> Page<'_> {
    let mut b = String::new();

    b.push_str("<div class=\"page-head\"><div class=\"wrap\">\n");
    b.push_str("<h1>Resources</h1>\n");
    b.push_str("<p class=\"lede\">Issue trackers, Git tutorials and communities that stay useful long after your first contribution.</p>\n");
    b.push_str("</div></div>\n");

    b.push_str("<section class=\"section\" style=\"border-top:0\"><div class=\"wrap\">\n");
    b.push_str("<div class=\"cols-flow\">\n");
    for sec in &site.resources {
        let _ = write!(b, "<div id=\"{}\">\n", esc(&sec.slug));
        let _ = write!(b, "<h2 style=\"font-size:18px;margin-bottom:14px\">{}</h2>\n", esc(&sec.section));
        b.push_str("<ul class=\"stack\">\n");
        for it in &sec.items {
            b.push_str("<li>\n");
            let _ = write!(
                b,
                "<div class=\"item-name\"><a href=\"{}\" rel=\"noopener\">{}{}</a></div>\n",
                esc(&it.url),
                esc(&it.name),
                icon::EXTERNAL
            );
            if let Some(d) = &it.description {
                let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc(d));
            }
            b.push_str("</li>\n");
        }
        b.push_str("</ul>\n</div>\n");
    }
    b.push_str("</div>\n</div></section>\n");

    // Contributing + disclaimer.
    b.push_str("<section class=\"section\"><div class=\"wrap\">\n");
    b.push_str("<h2>Keeping this list accurate</h2>\n");
    b.push_str("<p class=\"section-lede\">Program dates, eligibility and stipends change every cycle. If you spot something stale, a pull request takes two minutes.</p>\n");
    b.push_str("<div class=\"disclaimer\">\n");
    b.push_str("<strong>Verify before you apply.</strong> Everything here is community-maintained and can fall out of date between cycles. Always confirm dates, stipend amounts and eligibility on the program's official website before submitting an application.\n");
    b.push_str("</div>\n");
    b.push_str("<ul class=\"stack\">\n");
    for (name, desc, url) in [
        (
            "Add or update a program",
            "Edit data/programs.yml and open a pull request. Include an official link that confirms the dates.",
            format!("{REPO_URL}/blob/main/data/programs.yml"),
        ),
        (
            "Report an outdated entry",
            "Open an issue with the program name and what changed.",
            format!("{REPO_URL}/issues/new"),
        ),
    ] {
        b.push_str("<li>\n");
        let _ = write!(
            b,
            "<div class=\"item-name\"><a href=\"{}\" rel=\"noopener\">{}</a></div>\n",
            esc(&url),
            esc(name)
        );
        let _ = write!(b, "<p class=\"item-desc\">{}</p>\n", esc(desc));
        b.push_str("</li>\n");
    }
    b.push_str("</ul>\n</div></section>\n");

    Page {
        title: "Resources",
        description: "Tools and guides for open source contributors: beginner issue trackers, Git tutorials, open source education and communities for support.",
        path: "/resources/",
        nav: "resources",
        body: b,
        json_ld: None,
    }
}
