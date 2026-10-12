use std::sync::LazyLock;

use regex::Regex;

use super::{About, Context, DocumentLinter, Example};
use crate::issue::Finding;
use crate::markdown::Document;
use crate::{Error, FIRST_RELEASE};

static ALERT: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^[\t\n\f\r ]*>[\t\n\f\r ]*\[!([A-Za-z]+)]").expect("valid pattern"));

pub(super) struct Alerts;

impl DocumentLinter for Alerts {
	fn about(&self) -> About {
		About {
			name: "alerts",
			description: "an alert tag is one of the allowed types and is followed by an empty `>` line",
			since: FIRST_RELEASE,
			example: Example::new(
				"> [!INFO]\n> Restart the service after a change.\n",
				"> [!NOTE]\n>\n> Restart the service after a change.\n",
			),
		}
	}

	fn check(&self, c: &Context<'_>, path: &str, doc: &Document) -> Result<Vec<Finding>, Error> {
		let allowed = &c.settings.alerts.types;
		let mut out = Vec::new();
		for (i, line) in doc.lines.iter().enumerate() {
			let Some(kind) = ALERT
				.captures(&line.text)
				.filter(|_| !line.code)
				.map(|m| m[1].to_owned())
			else {
				continue;
			};
			if !allowed.contains(&kind) {
				let message = format!("unknown alert type [!{kind}] (allowed: {})", allowed.join(", "));
				out.push(Finding::new(path, i + 1, message));
			}
			if doc.lines.get(i + 1).is_none_or(|next| next.text.trim() != ">") {
				out.push(Finding::new(
					path,
					i + 1,
					"alert tag must be followed by an empty > line",
				));
			}
		}
		Ok(out)
	}
}
