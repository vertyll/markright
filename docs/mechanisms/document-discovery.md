# Document discovery

The documents are the `*.md` files `git ls-files --cached --others --exclude-standard` lists from the root: tracked,
or new and not yet added, but never ignored. A listed file that is gone from disk, a deletion not yet committed, is
left out. Outside a Git repository, or without Git, every `*.md` under the root is a document.

Either way, a file under a directory of build output or dependencies is skipped, at any depth. Git usually ignores
those already; skipping them covers a repository that does not. Anything else a repository wants left alone is an
exclusion in [the configuration](../configuration.md#exclusions), for the formatters, the linters or both.

Documents are read and written through a directory opened at the root with `cap-std`, so a fix never writes outside
it, even through a symbolic link. A document must be UTF-8; one that is not stops the run with an error naming it.

The same set is what [`index`](../rules.md#linters) expects the index document to link, so a document in a skipped
directory never needs listing.
