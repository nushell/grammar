# Nushell grammar

Nushell has no official grammar. The language is whatever `nu-parser` accepts. This repository
writes that down as a BNF grammar, derived rule by rule from nu-parser's source. It also holds
15,235 Nushell inputs and nu-parser's verdict on each one, so you can check the grammar against
the parser and notice when the parser changes.

The grammar describes nu-parser only. Other parsers, formatters and editor tools can use it and
the corpus as a reference.

## What's here

| path | what it is |
| --- | --- |
| [`grammar.md`](grammar.md) | The grammar, with the reasoning behind each rule. This is the file to edit. |
| [`grammar.bnf`](grammar.bnf) | Every rule from `grammar.md` in one file. Generated. |
| [`grammar.ebnf`](grammar.ebnf) | The same rules in ISO 14977 EBNF. Generated. `ebnf2railroad` and the VS Code EBNF extension read it. |
| [`grammar.html`](grammar.html) | Railroad diagrams rendered from `grammar.ebnf`. Open it in a browser. |
| [`corpus/`](corpus) | Nushell inputs, plus nu-parser's verdict for each in `corpus/expected/`. |
| [`harness/`](harness) | A Rust crate that runs nu-parser over the corpus and compares the verdicts. |
| [`tools/`](tools) | `gen-grammar.nu` writes the `.bnf` and `.ebnf` files. `extract-corpus.nu` collects the snippet corpus. |
| [`pest/`](pest) | The repository's original 2022 pest grammar and its test crate. Nobody maintains it. Section 9.1 of `grammar.md` lists where it differs from nu-parser. |

## Reading the grammar

Section 0 of `grammar.md` explains the notation.

Some rules are syntactic, and the text alone decides them. Others are shape directed. To decide
those, nu-parser needs a command's signature, the declarations in scope, constant evaluation,
types or files. Section 8 lists the shape-directed rules. Section 9 lists inputs whose verdict
depends on that information.

A `; nu:` comment under a rule points at the nu-parser code that decides it. Paths are relative
to `crates/nu-parser/src` in the nushell repository. Line numbers match commit `d7cb4e589` for
most rules, and commits up to `706259d6f` for rules revised later. The first paragraph of
`grammar.md` names these commits.

The verdicts in `corpus/expected/` come from a newer commit, `a4819256a`, pinned in
`harness/Cargo.toml`. A `; nu:` line number can be a few lines off from that commit until someone
re-reads the rule against it. Every cited file and function still exists there.

## Checking the grammar

You need a recent `nu` and Rust 1.97.1 or newer, which is nushell's minimum at the pinned
commit. The lint and the diagrams also need `ebnf2railroad`, which you can install with
`npm install -g ebnf2railroad`.

```nushell
nu tools/gen-grammar.nu --check                 # grammar.bnf and grammar.ebnf match grammar.md
ebnf2railroad --lint grammar.ebnf --no-target   # the EBNF is well formed
cd harness; cargo test                          # nu-parser's verdicts match corpus/expected
```

The first `cargo test` builds nushell's command crates and takes a few minutes. After that the
harness parses all 15,235 inputs in about two seconds.

## The corpus

| set | inputs | what it is |
| --- | ---: | --- |
| `corpus/fixtures/accept/` | 1,096 | One construct per file that the syntactic rules accept. |
| `corpus/fixtures/reject/` | 525 | One construct per file that the syntactic rules reject. |
| `corpus/files/` | 14 | Real files: std modules, the default config and env, `kitchen_sink.nu` and others. |
| `corpus/snippets/` | 2,808 | All 1,729 built-in command examples and all 1,079 Nushell code blocks in the book. |
| `corpus/generated/flag-values/` | 2,720 | 48 keyword statements with a help flag or a flag value in each slot, plus switch values and short-flag batches. |
| `corpus/generated/alias-missing-positional/` | 7,172 | Every one-line fixture, and each prefix of its words, as an alias target. |
| `corpus/generated/range-bounds/` | 900 | Ranges whose bound is a group or a raw string, as a statement, an argument and a value. |

The harness fails if any verdict differs from its line in `corpus/expected/`. It also fails if
nu-parser accepts a reject fixture, since anything the syntactic rules reject, nu-parser rejects
too.

The reverse doesn't hold. 158 accept fixtures fail in nu-parser because they need something the
text doesn't carry: a command's signature, a type, a constant, or a module, file or plugin that
isn't there. Section 9 of `grammar.md` covers these cases. The 147 fixtures named
`*-needs-context.nu` depend on that information on purpose. 143 of them fail, and the other four
pass with the context nu provides. The remaining 15 failures come from a missing module, file or
plugin, a command's input or output type, or an attribute evaluated at parse time.

Each line of a `corpus/expected/*.txt` file is a key, a tab and a verdict. The key is the input's
path below its set directory, or `<list>.json#<index>` for a snippet list. The verdict is `ok`,
`error <Variant>` or `error <Variant>: <label>`. An error verdict names the first `ParseError`
nu-parser reports, the same one `nu-check` shows, and the text of its first label. Verdicts leave
out spans, so a change to the standard library doesn't rewrite every line.

`harness/.cargo/config.toml` points `HOME` and the XDG directories at paths that don't exist, so
inputs that read `~` or `$nu.config-path` get the same verdict on every machine. Cargo only reads
that file when you run it from `harness/`. The expected verdicts were generated on macOS.

## Keeping it current

### Changing a rule

Edit `grammar.md`, then regenerate the derived files.

```nushell
nu tools/gen-grammar.nu
ebnf2railroad --lint grammar.ebnf --no-target
ebnf2railroad --title "Nushell Grammar" grammar.ebnf -o grammar.html
```

Don't edit `grammar.bnf`, `grammar.ebnf` or `grammar.html` by hand. `gen-grammar.nu --check`
fails when the `.bnf` or `.ebnf` file is stale. Nothing checks `grammar.html`, so rerun
`ebnf2railroad` whenever the EBNF changes.

### Adding coverage

Put the input in `corpus/fixtures/accept/<area>/<name>.nu` or `corpus/fixtures/reject/...`, one
construct per file. Run `UPDATE_EXPECTED=1 cargo test` in `harness/` and read the new line in
`corpus/expected/fixtures.txt`. If the verdict surprises you, `open --raw <file> | nu-check` gives
a quick second opinion.

### Moving to a newer nushell

1. Copy nushell's `Cargo.lock` at the new commit over `harness/Cargo.lock`, so the harness
   builds with the same dependency versions `nu` does.
2. Change `rev` on every nushell line of `harness/Cargo.toml`.
3. Run `UPDATE_EXPECTED=1 cargo test` in `harness/`, then `git diff corpus/expected`.
4. Each changed verdict means nu-parser changed. Update the affected rules and their `; nu:`
   lines in `grammar.md`, along with the commit and date in its first paragraph. Then regenerate.
5. To pick up new command examples and book code blocks, run
   `nu tools/extract-corpus.nu --nushell <checkout at that commit>`.

The flag-value and range sets come from deterministic scripts next to their inputs. For example,
`python3 corpus/generated/range-bounds/gen.py corpus/generated/range-bounds/ranges.json`
rewrites the range set. The alias targets have no script. They were derived from the fixtures
once.

## License

MIT, see [LICENSE](LICENSE).
