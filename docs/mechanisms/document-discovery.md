# Document discovery

The documents are the `*.md` files `git ls-files --cached --others --exclude-standard` lists from the root: tracked,
or new and not yet added, but never ignored. A listed file that is gone from disk, a deletion not yet committed, is
left out. Outside a Git repository, or without Git, every `*.md` under the root is a document.

Either way, a file under a directory of build output or dependencies is skipped, at any depth. Git usually ignores
those already; skipping them covers a repository that does not. A file matching `documents.exclude` is no document
either: [Documents](../configuration.md#documents). Anything a repository wants checked but not formatted, or the
other way round, is an exclusion in [the configuration](../configuration.md#exclusions).

The root is the nearest directory, from the current one up, that holds a configuration file or `.git`, unless `--root`
names it. Every document under it is read on every run, so links and anchors between documents resolve; the paths a
command is given, or the current directory, only choose which of them are reported and rewritten.

Documents are read and written through a directory opened at the root with `cap-std`, so a fix never writes outside
it, even through a symbolic link. A document must be UTF-8; one that is not stops the run with an error naming it.

The same set is what [`index`](../rules.md#linters) expects the index document to link, so a document in a skipped
directory never needs listing.
