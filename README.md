# Nushell grammar

Nushell has no official grammar. The language is whatever `nu-parser` accepts. This repository
writes that down as a BNF grammar, derived rule by rule from nu-parser's source. It also holds
15,230 Nushell inputs and nu-parser's verdict on each one, so you can check the grammar against
the parser and see when the parser moves.

The grammar describes nu-parser and nothing else. Other projects (parsers, formatters, editor
tooling) can use it and its corpus as a reference.

## What's here

| path | what it is |
| --- | --- |
| [`grammar.md`](grammar.md) | The grammar, with the discussion behind each rule. This is the file to edit. |
| [`grammar.bnf`](grammar.bnf) | The rules of `grammar.md` in one piece, generated. |
| [`grammar.ebnf`](grammar.ebnf) | The same rules in ISO 14977 EBNF, generated. `ebnf2railroad` and the VS Code EBNF extension read it. |
| [`grammar.html`](grammar.html) | Railroad diagrams rendered from `grammar.ebnf`. Open it in a browser. |
| [`corpus/`](corpus) | Nushell inputs, and nu-parser's verdict for each in `corpus/expected/`. |
| [`harness/`](harness) | A Rust crate that runs nu-parser over the corpus and compares the verdicts. |
| [`tools/`](tools) | `gen-grammar.nu` writes the `.bnf` and `.ebnf` files. `extract-corpus.nu` collects the snippet corpus. |
| [`pest/`](pest) | The repository's original pest grammar from 2022 and its test crate. It is no longer maintained. Section 9.1 of `grammar.md` lists where it differs from nu-parser. |

## Reading the grammar

Section 0 of `grammar.md` explains the notation. Two things matter most.

Some rules are syntactic and the text alone decides them. Others are shape directed. For those,
nu-parser needs a command's signature, the declarations in scope, constant evaluation, types or
files. Section 8 lists the shape-directed rules, and section 9 lists inputs whose verdict depends
on that information.

A `; nu:` comment under a rule names the place in nu-parser that decides it. Paths are relative
to `crates/nu-parser/src` in the nushell repository, at the commit named in the first paragraph
of `grammar.md`.

## Checking the grammar

You need a recent `nu` and a Rust toolchain no older than nushell's minimum at the pinned
revision (1.97.1). `ebnf2railroad` (`npm install -g ebnf2railroad`) is only needed for the lint
and the diagrams.

```nushell
nu tools/gen-grammar.nu --check                 # grammar.bnf and grammar.ebnf match grammar.md
ebnf2railroad --lint grammar.ebnf --no-target   # the EBNF is well formed
cd harness; cargo test                          # nu-parser's verdicts match corpus/expected
```

The first `cargo test` builds nushell's command crates, which takes a few minutes. After that the
harness parses all 15,230 inputs in about a second.

## The corpus

| set | inputs | what it is |
| --- | ---: | --- |
| `corpus/fixtures/accept/` | 1,092 | One construct per file, written so that the grammar's syntactic rules accept it. |
| `corpus/fixtures/reject/` | 524 | One construct per file that the syntactic rules refuse. |
| `corpus/files/` | 14 | Real files: std modules, the default config and env, `kitchen_sink.nu` and others. |
| `corpus/snippets/` | 2,808 | Every built-in command example (1,729) and every Nushell code block in the book (1,079). |
| `corpus/generated/flag-values/` | 2,720 | 48 keyword statements with a help flag or a flag value in each slot, plus switch values and short-flag batches. |
| `corpus/generated/alias-missing-positional/` | 7,172 | Every one-line fixture, and each prefix of its words, as an alias target. |
| `corpus/generated/range-bounds/` | 900 | Ranges whose bound is a group or a raw string, as a statement, an argument and a value. |

The harness checks two things. Every verdict must match its line in `corpus/expected/`. And
nu-parser must reject every reject fixture, because anything the syntactic rules refuse, nu
refuses too. The converse does not hold. 157 accept fixtures fail in nu-parser because of
something the text doesn't carry: a command's signature, a type, a constant, or a module, file
or plugin that isn't there. Section 9 of `grammar.md` covers these cases, and 142 of the 157
have names ending in `-is-consumer`.

Each line of a `corpus/expected/*.txt` file is a key, a tab and a verdict. The key is the input's
path below its set directory, or `<list>.json#<index>` for a snippet list. The verdict is `ok`,
`error <Variant>` or `error <Variant>: <label>`. That is the first `ParseError` nu-parser
reports, as `nu-check` would, with the text of its first label. Spans are left out, so a change
to the standard library doesn't touch every line.

## Keeping it current

### Changing a rule

Edit `grammar.md`, then regenerate the derived files.

```nushell
nu tools/gen-grammar.nu
ebnf2railroad --lint grammar.ebnf --no-target
ebnf2railroad --title "Nushell Grammar" grammar.ebnf -o grammar.html
```

Never edit `grammar.bnf`, `grammar.ebnf` or `grammar.html` by hand. `gen-grammar.nu --check`
fails when they are stale.

### Adding coverage

Put the input in `corpus/fixtures/accept/<area>/<name>.nu` or `corpus/fixtures/reject/...`, one
construct per file. Run `UPDATE_EXPECTED=1 cargo test` in `harness/` and read the new line in
`corpus/expected/fixtures.txt`. If the verdict surprises you, `open --raw <file> | nu-check` is a
quick second opinion.

### Moving to a newer nushell

1. Copy nushell's `Cargo.lock` at the new revision over `harness/Cargo.lock`, so the harness
   builds with the same dependency versions `nu` does.
2. Change `rev` on every nushell line of `harness/Cargo.toml`.
3. Run `UPDATE_EXPECTED=1 cargo test` in `harness/`, then `git diff corpus/expected`.
4. Each changed verdict means nu-parser changed. Update the affected rules and their `; nu:`
   lines in `grammar.md`, and the commit and date in its first paragraph. Then regenerate.
5. To pick up new command examples and book code blocks, run
   `nu tools/extract-corpus.nu --nushell <checkout at that revision>`.

The generated sets come from deterministic scripts next to their inputs. For example,
`python3 corpus/generated/range-bounds/gen.py corpus/generated/range-bounds/ranges.json`
rewrites the range set. The alias targets were derived from the fixtures once and have no
script.

## License

MIT, see [LICENSE](LICENSE).
