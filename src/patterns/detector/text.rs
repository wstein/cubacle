//! The report as short human text.

use std::fmt;

use super::{Report, Symmetry};

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(f, false)
    }
}

/// The report with one line per symmetry operation and its colour renaming.
pub struct Detailed<'a>(pub(super) &'a Report);

impl fmt::Display for Detailed<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.write(f, true)
    }
}

impl Report {
    fn write(&self, f: &mut fmt::Formatter<'_>, verbose: bool) -> fmt::Result {
        write_matches(f, &self.matches)?;
        writeln!(f, "patternness: {} %", self.patternness)?;
        write_fingerprint(f, &self.fingerprint)?;
        write_subgroups(f, &self.subgroups)?;
        write_symmetry(f, &self.symmetry, verbose)
    }

    /// The report with every symmetry operation listed (`identify -v`).
    pub fn detailed(&self) -> Detailed<'_> {
        Detailed(self)
    }
}

/// A short plural for the class counts: `3 edge mirrors`.
fn short_class(class: &str) -> &'static str {
    match class {
        "identity" => "identity",
        "quarter turn about a face axis" => "face quarter turn",
        "half turn about a face axis" => "face half turn",
        "half turn about an edge axis" => "edge half turn",
        "third turn about a corner axis" => "corner third turn",
        "inversion through the centre" => "inversion",
        "mirror parallel to a face" => "face mirror",
        "mirror through opposite edges" => "edge mirror",
        "quarter turn and mirror" => "quarter-turn mirror",
        _ => "sixth-turn mirror",
    }
}

fn write_symmetry(f: &mut fmt::Formatter<'_>, symmetry: &Symmetry, verbose: bool) -> fmt::Result {
    let order = symmetry.order();
    writeln!(
        f,
        "symmetry: {order} of 48 operations fix it ({} rotations, {} mirrored)",
        symmetry.rotations(),
        order - symmetry.rotations()
    )?;
    writeln!(
        f,
        "  {} of them keep every colour, {} rename colours",
        symmetry.colour_preserving(),
        order - symmetry.colour_preserving()
    )?;
    let mut classes: Vec<(&str, Vec<&super::Operation>)> = Vec::new();
    for operation in &symmetry.operations {
        match classes.iter_mut().find(|(c, _)| *c == operation.class) {
            Some((_, list)) => list.push(operation),
            None => classes.push((operation.class, vec![operation])),
        }
    }
    let counts: Vec<String> = classes
        .iter()
        .map(|(class, list)| {
            let word = short_class(class);
            match (list.len(), word) {
                (1, _) | (_, "identity" | "inversion") => format!("{} {word}", list.len()),
                (n, _) => format!("{n} {word}s"),
            }
        })
        .collect();
    writeln!(f, "  classes: {}", counts.join(", "))?;
    if !verbose {
        return Ok(());
    }
    for (class, operations) in classes {
        let mut axes: Vec<(String, usize)> = Vec::new();
        for o in operations.iter().filter(|o| !o.axis.is_empty()) {
            let text = match o.colour_swap.as_str() {
                "" => o.axis.clone(),
                swap => format!("{} renaming {swap}", o.axis),
            };
            match axes.iter_mut().find(|(t, _)| *t == text) {
                Some((_, n)) => *n += 1,
                None => axes.push((text, 1)),
            }
        }
        let swapped = operations.iter().filter(|o| !o.keeps_colours()).count();
        if axes.is_empty() || axes.len() > 8 {
            let note = if swapped == 0 {
                String::new()
            } else {
                format!(", {swapped} renaming colours")
            };
            writeln!(f, "  {} x {class}{note}", operations.len())?;
        } else {
            let list: Vec<String> = axes
                .iter()
                .map(|(t, n)| {
                    if *n > 1 {
                        format!("{t} (x{n})")
                    } else {
                        t.clone()
                    }
                })
                .collect();
            writeln!(f, "  {} x {class}: {}", operations.len(), list.join(", "))?;
        }
    }
    Ok(())
}

fn write_matches(f: &mut fmt::Formatter<'_>, matches: &[super::Match]) -> fmt::Result {
    if matches.is_empty() {
        return writeln!(f, "pattern: no library match");
    }
    for m in matches {
        writeln!(f, "pattern: {} ({}), {}", m.name, m.id, m.relation)?;
    }
    Ok(())
}

fn write_fingerprint(f: &mut fmt::Formatter<'_>, fingerprint: &super::Fingerprint) -> fmt::Result {
    writeln!(f, "fingerprint: {}", fingerprint.code())?;
    for orbit in &fingerprint.orbits {
        let what = match orbit.name.split_once(' ') {
            Some(("wings", depth)) => format!("wings (depth {depth})"),
            _ => orbit.name.clone(),
        };
        let unit = if orbit.tag.is_some() {
            "centre stickers"
        } else {
            "pieces"
        };
        let mut parts = vec![format!("{} {unit}", orbit.pieces)];
        if orbit.tag.is_some() {
            parts.push(format!("{} misplaced", orbit.displaced));
        } else {
            parts.push(format!("{} displaced", orbit.displaced));
            let word = if orbit.name == "corners" {
                "twisted"
            } else {
                "flipped"
            };
            parts.push(format!("{} {word}", orbit.oriented_wrong));
        }
        match &orbit.cycles {
            Some(c) if !c.is_empty() => {
                let list: Vec<String> = c.iter().map(ToString::to_string).collect();
                parts.push(format!("cycles of length {}", list.join(", ")));
            }
            _ => {}
        }
        writeln!(f, "  {what}: {}", parts.join(", "))?;
    }
    Ok(())
}

fn write_subgroups(f: &mut fmt::Formatter<'_>, subgroups: &[super::Membership]) -> fmt::Result {
    for m in subgroups {
        let (word, why) = match m.verdict {
            super::Verdict::Member => ("yes", String::new()),
            super::Verdict::NotMember => ("no", format!(": {}", m.why)),
            super::Verdict::Possible => ("possible", format!(" ({})", m.why)),
        };
        writeln!(f, "in {}: {word}{why}", m.name)?;
    }
    Ok(())
}
