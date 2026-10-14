# Contributing

Thank you for helping with markright. This document says how a change gets in; [Development](docs/development.md)
says how the code is built, checked and released.

## Before you start

- **A bug:** open an issue with the document, or the smallest part of it, that shows the problem, the command you
  ran, what markright printed or changed, and what you expected. Say which version `markright version` prints.
- **A new rule or setting:** open an issue first. Describe what it reports or rewrites, with a document it changes
  and the same document after, and why a renderer reads the two alike, for a formatter, or why the first is a
  mistake, for a linter. A rule that would report much of what people write on purpose fits better as a setting of an
  existing one, or not at all.
- **A fix to the documentation:** a pull request is enough.

## Making the change

1. Fork the repository and branch from `main`.
2. Make the change with its tests. A new formatter or linter follows
   [Development: Adding a formatter or linter](docs/development.md#adding-a-formatter-or-linter).
3. Update the documents the change touches: a new term goes in [GLOSSARY.md](GLOSSARY.md), a new document in
   [CONTENTS.md](CONTENTS.md), and a specification the tool starts to follow in [STANDARDS.md](STANDARDS.md).
4. Run every check, as CI does:

   ```bash
   bash scripts/check.sh
   ```

   When the change touches the site, build it and check its links as well:
   [Development: The documentation site](docs/development.md#the-documentation-site).

## Commits and pull requests

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/): a type, a colon and
what the commit does, in the imperative and in lower case.

| Type       | For                                                |
|------------|----------------------------------------------------|
| `feat`     | a new rule, setting, command or output             |
| `fix`      | a rule that reports or rewrites what it should not |
| `docs`     | the documentation only                             |
| `refactor` | a change of the code that changes no behaviour     |
| `test`     | tests only                                         |
| `build`    | the toolchain, dependencies, scripts and CI        |

A change that breaks configurations that work today adds `!` after the type, as `feat!:`, and says what to change in
its body. A pull request does one thing, says why, and passes CI before it is reviewed.

## License

By contributing, you agree that your contribution is released under the [MIT License](LICENSE), as the rest of
markright.
