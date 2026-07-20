# Contributing

This list is only useful if it stays accurate, so corrections are as valuable
as additions. Reporting a stale deadline takes two minutes and helps everyone
reading the site.

## Where the data lives

Everything the site shows comes from YAML in `data/`:

| File | What it holds |
| --- | --- |
| `data/programs.yml` | Mentorships, fellowships and internships |
| `data/competitions.yml` | Hacktoberfest, 24 Pull Requests and similar |
| `data/timeline.yml` | Dated events for the calendar page |
| `data/resources.yml` | Link collections on the Resources page |
| `data/guide.yml` | Content for the Start here page |

Edit those files, not `dist/`, which is generated output and is overwritten on
every build.

`Readme.md` is maintained separately by hand. You do not need to update it when
you change the data.

## Adding a program

Copy an existing entry in `data/programs.yml` and fill it in:

```yaml
- name: Google Summer of Code
  short_name: GSoC            # optional, used where space is tight
  url: https://summerofcode.withgoogle.com/
  org: Google
  timeline: "Org Apps: Jan 19 - Feb 3 - Contributors: Mar 16 - Mar 31"
  stipend:
    min: 1500
    max: 6600
    currency: USD             # required whenever min/max are set
    note: 12-22 week projects  # optional detail shown under the amount
  eligibility: 18+ years, open to all
  tags: [paid, global, beginner-friendly]
```

Only `name`, `url` and `org` are strictly required. For programs that do not
pay a fixed amount, use `note` instead of `min`/`max`:

```yaml
  stipend:
    note: Varies / may be paid depending on program
```

For programs offering no money, say so explicitly:

```yaml
  stipend:
    unpaid: true
    note: Certificate and swag
```

### Tags

| Tag | Meaning |
| --- | --- |
| `paid` | Offers money, whatever the amount |
| `unpaid` | Offers no money |
| `global` | Open to applicants worldwide |
| `students` | Restricted to students |
| `beginner-friendly` | Suitable for a first open source contribution |
| `underrepresented` | Aimed at underrepresented groups in tech |

`paid` and `unpaid` are mutually exclusive; setting both fails the build.

## Verify before you submit

Every date, stipend and eligibility line should come from the program's own
website. Blog posts and aggregator sites go stale and are frequently wrong.

Link your source in the pull request. Entries that cannot be checked are hard
to merge in good conscience, since someone may plan around them.

## Build it locally

```bash
./build.sh --serve      # http://localhost:8000
```

Needs `cargo`, plus `python3` for `--serve`.

The generator validates the data and refuses to build on real contradictions:

- two programs with the same name
- a stipend amount with no currency, or `min` above `max`
- a program tagged both `paid` and `unpaid`
- a URL that is not absolute

It warns, but still builds, when an entry is merely incomplete (no timeline, no
eligibility, a timeline event naming a program that is not in the list). If CI
fails on your pull request, the error message names the offending entry.

## Removing a program

Prefer removing entries that have clearly ended over leaving them to rot. A
list of dead programs is worse than a shorter accurate one. If you are unsure
whether a program is still running, open an issue rather than deleting it.
