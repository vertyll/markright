# Security

## Supported versions

Fixes go into the latest release only: a vulnerability is fixed in a new patch version of the latest minor version,
and older versions are not patched. Upgrading is raising the version a repository pins.

## Reporting a vulnerability

Report it privately, never in a public issue: open
[Report a vulnerability](https://github.com/vertyll/markright/security/advisories/new) on the repository's Security
tab. Say which version `markright version` prints, and give the steps, or the smallest document and configuration,
that show the problem.

The report is answered within a week. Once it is confirmed, a fix is prepared in a private fork, released as a new
version, and the advisory is published with it, naming who reported the problem unless they ask otherwise.

## What counts

markright reads the documents and the configuration of a repository, and writes only to them and to the report
files the configuration names. A vulnerability is anything that breaks that, such as:

- reading or writing a file outside the repository it runs in, other than the report files its configuration names;
- a document or configuration that makes a run hang, or use memory without bound, so a CI job never ends;
- a crash on a document a renderer reads without trouble;
- in the playground of the documentation site, a document that runs code in the page.

A rule that reports what it should not, or misses what it should, is a bug: open an issue for it, as
[CONTRIBUTING.md](CONTRIBUTING.md) says.
