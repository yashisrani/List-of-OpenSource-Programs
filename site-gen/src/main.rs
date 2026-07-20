// Static-site generator for the open source programs list.
//
// Design goals:
//  - DATA-DRIVEN: everything the site shows comes from the YAML files in
//    `data/`. Adding a program is a pull request against one file, and the site
//    picks it up on the next build. Readme.md is left alone on purpose; it is
//    the repo's front door and is maintained by hand.
//  - VALIDATED: the generator refuses to build on contradictory data
//    (duplicate programs, a stipend with no currency, min above max) and warns
//    on merely incomplete entries. See validate.rs for the full policy.
//  - SEO-FIRST: every page is fully server-rendered with real content in the
//    initial HTML, unique title/description/canonical, Open Graph tags and
//    JSON-LD, plus sitemap.xml and robots.txt. Someone searching for a specific
//    program should land here.
//  - NO RUNTIME: output is plain HTML/CSS/JS with no build step for the client
//    and no backend. Filtering is progressive enhancement over the rendered list.
//
// Output layout (all under `dist/`):
//   dist/index.html          program list with search and tag filters
//   dist/timeline/           month-by-month 2026 calendar
//   dist/start-here/         first-time contributor guide
//   dist/resources/          link collections
//   dist/assets/app.css | app.js
//   dist/favicon.svg | sitemap.xml | robots.txt

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

mod model;
mod render;
mod validate;

use model::{Guide, Program, ResourceSection, Site, Timeline};
use render::{Page, REPO_URL, SITE_URL};

fn read_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Box<dyn Error>> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    serde_yaml::from_str(&raw).map_err(|e| format!("cannot parse {}: {e}", path.display()).into())
}

fn load(data: &Path) -> Result<Site, Box<dyn Error>> {
    let programs: Vec<Program> = read_yaml(&data.join("programs.yml"))?;
    let competitions: Vec<Program> = read_yaml(&data.join("competitions.yml"))?;
    let timeline: Timeline = read_yaml(&data.join("timeline.yml"))?;
    let resources: Vec<ResourceSection> = read_yaml(&data.join("resources.yml"))?;
    let guide: Guide = read_yaml(&data.join("guide.yml"))?;
    Ok(Site {
        programs,
        competitions,
        timeline,
        resources,
        guide,
    })
}

/// Write a page to `dist/<path>/index.html` (or `dist/index.html` for "/").
fn write_page(dist: &Path, page: &Page) -> Result<(), Box<dyn Error>> {
    let dir = if page.path == "/" {
        dist.to_path_buf()
    } else {
        dist.join(page.path.trim_matches('/'))
    };
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("index.html"), render::shell(page))?;
    Ok(())
}

/// Favicon: the same square mark the header uses, so the tab matches the page.
const FAVICON: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" rx="7" fill="#2f6b45"/><rect x="9" y="9" width="14" height="14" rx="3" fill="#ecf3ee"/></svg>"##;

fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::current_dir()?;
    let data = match std::env::args().nth(1) {
        Some(a) => PathBuf::from(a),
        None => root.join("data"),
    };
    if !data.is_dir() {
        return Err(format!(
            "data directory not found at {}. Run this from the repo root.",
            data.display()
        )
        .into());
    }
    let dist = root.join("dist");

    eprintln!("▸ Reading {}", data.display());
    let site = load(&data)?;

    // Validate before writing anything, so a failed build never leaves a
    // half-generated dist/ behind.
    let report = validate::check(&site);
    for w in &report.warnings {
        eprintln!("  warning: {w}");
    }
    if !report.errors.is_empty() {
        for e in &report.errors {
            eprintln!("  error: {e}");
        }
        return Err(format!(
            "{} data error(s) found; fix the YAML in {} and rebuild",
            report.errors.len(),
            data.display()
        )
        .into());
    }

    let total = site.programs.len() + site.competitions.len();
    eprintln!(
        "  {} programs, {} competitions, {} timeline events",
        site.programs.len(),
        site.competitions.len(),
        site.timeline.events.len()
    );

    if dist.exists() {
        fs::remove_dir_all(&dist)?;
    }
    fs::create_dir_all(dist.join("assets"))?;

    let pages = [
        render::home(&site),
        render::timeline(&site),
        render::start_here(&site),
        render::resources(&site),
    ];
    for p in &pages {
        write_page(&dist, p)?;
    }

    // Static assets, inlined at compile time so the binary is self-contained
    // and `cargo run` from anywhere still produces a complete site.
    fs::write(
        dist.join("assets/app.css"),
        include_str!("../web/app.css"),
    )?;
    fs::write(dist.join("assets/app.js"), include_str!("../web/app.js"))?;
    fs::write(dist.join("favicon.svg"), FAVICON)?;

    // sitemap.xml
    let mut sitemap = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for p in &pages {
        let loc = if p.path == "/" {
            format!("{SITE_URL}/")
        } else {
            format!("{SITE_URL}{}", p.path)
        };
        let priority = if p.path == "/" { "1.0" } else { "0.8" };
        sitemap.push_str(&format!(
            "  <url><loc>{loc}</loc><changefreq>weekly</changefreq><priority>{priority}</priority></url>\n"
        ));
    }
    sitemap.push_str("</urlset>\n");
    fs::write(dist.join("sitemap.xml"), sitemap)?;

    fs::write(
        dist.join("robots.txt"),
        format!("User-agent: *\nAllow: /\n\nSitemap: {SITE_URL}/sitemap.xml\n"),
    )?;

    eprintln!("✓ {} pages written to {}", pages.len(), dist.display());
    eprintln!("  {total} programs indexed. Source: {REPO_URL}");
    Ok(())
}
