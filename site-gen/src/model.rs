//! Data model mirroring the YAML files in `data/`.
//!
//! Every field the YAML may omit is an `Option` or defaults to empty, so a
//! sparse entry (a newly announced program with nothing but a name and a link)
//! still loads. Validation of what is *semantically* required happens in
//! `validate`, not here, so the error message can name the offending entry.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Stipend {
    pub min: Option<u64>,
    pub max: Option<u64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub unpaid: bool,
}

impl Stipend {
    /// Human-readable amount, e.g. "$1,500 - $6,600" or "₹1,00,000".
    /// Falls back to `note` when there is no numeric range to show.
    pub fn display(&self) -> String {
        match (self.min, self.max) {
            (Some(lo), Some(hi)) => {
                let sym = self.symbol();
                if lo == hi {
                    format!("{sym}{}", group(lo, self.is_inr()))
                } else {
                    format!(
                        "{sym}{} - {sym}{}",
                        group(lo, self.is_inr()),
                        group(hi, self.is_inr())
                    )
                }
            }
            _ => self
                .note
                .clone()
                .unwrap_or_else(|| if self.unpaid { "Unpaid".into() } else { "Varies".into() }),
        }
    }

    /// True when `display()` returns a real currency amount rather than prose.
    /// Drives whether the value is styled as a figure or as quiet supporting
    /// text.
    pub fn has_amount(&self) -> bool {
        self.min.is_some() && self.max.is_some()
    }

    /// Secondary line shown under the amount: the note when the amount already
    /// used the numbers, nothing when the note *was* the amount.
    pub fn sub_note(&self) -> Option<&str> {
        if self.min.is_some() && self.max.is_some() {
            self.note.as_deref()
        } else {
            None
        }
    }

    fn is_inr(&self) -> bool {
        self.currency.as_deref() == Some("INR")
    }

    fn symbol(&self) -> &'static str {
        match self.currency.as_deref() {
            Some("INR") => "₹",
            Some("EUR") => "€",
            _ => "$",
        }
    }

    /// Sort key for "highest paying first". Unpaid and unknown sink to the
    /// bottom. INR is converted at a deliberately rough rate purely for
    /// ordering; it is never displayed.
    pub fn sort_value(&self) -> u64 {
        let raw = self.max.or(self.min).unwrap_or(0);
        if self.is_inr() {
            raw / 84
        } else {
            raw
        }
    }
}

/// Digit grouping. Indian numbering (1,00,000) differs from western (100,000),
/// and showing ₹100,000 to an audience that reads lakhs looks wrong.
fn group(n: u64, indian: bool) -> String {
    let s = n.to_string();
    if !indian {
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                out.push(',');
            }
            out.push(c);
        }
        return out;
    }
    // Indian: last 3 digits, then groups of 2.
    if s.len() <= 3 {
        return s;
    }
    let (head, tail) = s.split_at(s.len() - 3);
    let mut parts = Vec::new();
    let mut rest = head;
    while rest.len() > 2 {
        let (h, t) = rest.split_at(rest.len() - 2);
        parts.push(t.to_string());
        rest = h;
    }
    if !rest.is_empty() {
        parts.push(rest.to_string());
    }
    parts.reverse();
    format!("{},{}", parts.join(","), tail)
}

#[derive(Debug, Deserialize)]
pub struct Program {
    pub name: String,
    #[serde(default)]
    pub short_name: Option<String>,
    pub url: String,
    pub org: String,
    #[serde(default)]
    pub timeline: Option<String>,
    /// Year the `timeline` string describes. Not rendered directly (the
    /// timeline text already spells out dates), but kept so the YAML can
    /// record which cycle a date belongs to for future filtering.
    #[serde(default)]
    #[allow(dead_code)]
    pub timeline_year: Option<u32>,
    #[serde(default)]
    pub stipend: Option<Stipend>,
    #[serde(default)]
    pub eligibility: Option<String>,
    /// Shown as an inline warning on the card. Used for things like the MLH
    /// region-availability caveat that a contributor would otherwise discover
    /// only after applying.
    #[serde(default)]
    pub caveat: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Competitions carry rewards instead of a stipend.
    #[serde(default)]
    pub rewards: Option<String>,
}

impl Program {
    pub fn display_name(&self) -> &str {
        &self.name
    }

    pub fn is_paid(&self) -> bool {
        self.tags.iter().any(|t| t == "paid")
    }

    pub fn stipend_sort(&self) -> u64 {
        self.stipend.as_ref().map(|s| s.sort_value()).unwrap_or(0)
    }
}

#[derive(Debug, Deserialize)]
pub struct TimelineEvent {
    pub month: u32,
    pub date: String,
    pub event: String,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub approx: bool,
}

#[derive(Debug, Deserialize)]
pub struct Timeline {
    pub year: u32,
    pub events: Vec<TimelineEvent>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceLink {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceSection {
    pub section: String,
    pub slug: String,
    pub items: Vec<ResourceLink>,
}

#[derive(Debug, Deserialize)]
pub struct BeginnerPick {
    pub program: String,
    pub why: String,
    pub getting_started: String,
}

#[derive(Debug, Deserialize)]
pub struct LanguageGuide {
    pub name: String,
    #[serde(default)]
    pub repos: Vec<ResourceLink>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NonCode {
    pub kind: String,
    pub description: String,
    #[serde(default)]
    pub links: Vec<ResourceLink>,
}

#[derive(Debug, Deserialize)]
pub struct LearningStep {
    pub weeks: String,
    pub goal: String,
}

#[derive(Debug, Deserialize)]
pub struct LabelHint {
    pub label: String,
    pub meaning: String,
}

#[derive(Debug, Deserialize)]
pub struct Guide {
    pub best_for_beginners: Vec<BeginnerPick>,
    pub languages: Vec<LanguageGuide>,
    pub non_code: Vec<NonCode>,
    pub learning_path: Vec<LearningStep>,
    pub labels: Vec<LabelHint>,
}

/// Everything the site is built from, loaded once.
pub struct Site {
    pub programs: Vec<Program>,
    pub competitions: Vec<Program>,
    pub timeline: Timeline,
    pub resources: Vec<ResourceSection>,
    pub guide: Guide,
}
