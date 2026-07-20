//! Build-time checks on the YAML data.
//!
//! This repo takes drive-by PRs adding a program, so the generator is the last
//! place a typo can be caught before it reaches the site. Errors fail the
//! build; warnings print and continue. The rule of thumb: anything that would
//! render as visibly broken or misleading is an error, anything merely
//! incomplete is a warning.

use std::collections::HashMap;

use crate::model::Site;

pub struct Report {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl Report {
    fn err(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }
    fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }
}

pub fn check(site: &Site) -> Report {
    let mut r = Report {
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let all: Vec<&crate::model::Program> =
        site.programs.iter().chain(site.competitions.iter()).collect();

    // Duplicate names. Readme.md listed Open Mainframe Project twice with
    // different stipends; that is exactly the drift this catches.
    let mut seen: HashMap<String, usize> = HashMap::new();
    for p in &all {
        let key = p.name.to_lowercase();
        *seen.entry(key).or_insert(0) += 1;
    }
    for (name, count) in &seen {
        if *count > 1 {
            r.err(format!("duplicate program name \"{name}\" appears {count} times"));
        }
    }

    for p in &all {
        if p.name.trim().is_empty() {
            r.err("a program has an empty name");
        }
        if !p.url.starts_with("https://") && !p.url.starts_with("http://") {
            r.err(format!("{}: url must be absolute, got \"{}\"", p.name, p.url));
        }
        if p.org.trim().is_empty() {
            r.warn(format!("{}: no org set", p.name));
        }
        if p.timeline.is_none() {
            r.warn(format!("{}: no timeline set", p.name));
        }
        if p.eligibility.is_none() {
            r.warn(format!("{}: no eligibility set", p.name));
        }

        // A program tagged `paid` with no stipend information tells the reader
        // nothing, and a program tagged both paid and unpaid is a mistake.
        let unpaid_tag = p.tags.iter().any(|t| t == "unpaid");
        if p.is_paid() && unpaid_tag {
            r.err(format!("{}: tagged both paid and unpaid", p.name));
        }
        if p.is_paid() && p.stipend.is_none() && p.rewards.is_none() {
            r.warn(format!("{}: tagged paid but has no stipend block", p.name));
        }
        if let Some(s) = &p.stipend {
            if let (Some(lo), Some(hi)) = (s.min, s.max) {
                if lo > hi {
                    r.err(format!("{}: stipend min {lo} exceeds max {hi}", p.name));
                }
            }
            if (s.min.is_some() || s.max.is_some()) && s.currency.is_none() {
                r.err(format!("{}: stipend has an amount but no currency", p.name));
            }
            if s.unpaid && (s.min.is_some() || s.max.is_some()) {
                r.err(format!("{}: stipend marked unpaid but carries an amount", p.name));
            }
        }
    }

    // Timeline months must be real, and every `program:` reference should
    // resolve to something in the list so cross-links do not dangle.
    let names: Vec<String> = all
        .iter()
        .flat_map(|p| {
            let mut v = vec![p.name.to_lowercase()];
            if let Some(s) = &p.short_name {
                v.push(s.to_lowercase());
            }
            v
        })
        .collect();

    for e in &site.timeline.events {
        if !(1..=12).contains(&e.month) {
            r.err(format!(
                "timeline event \"{}\" has month {}, expected 1-12",
                e.event, e.month
            ));
        }
        if let Some(prog) = &e.program {
            if !names.contains(&prog.to_lowercase()) {
                r.warn(format!(
                    "timeline event \"{}\" references unknown program \"{prog}\"",
                    e.event
                ));
            }
        }
    }

    // Guide picks reference programs by name too.
    for pick in &site.guide.best_for_beginners {
        if !names.contains(&pick.program.to_lowercase()) {
            r.warn(format!(
                "guide.best_for_beginners references unknown program \"{}\"",
                pick.program
            ));
        }
    }

    let mut slugs: HashMap<&str, usize> = HashMap::new();
    for s in &site.resources {
        *slugs.entry(s.slug.as_str()).or_insert(0) += 1;
    }
    for (slug, count) in slugs {
        if count > 1 {
            r.err(format!("duplicate resource section slug \"{slug}\""));
        }
    }

    r
}
