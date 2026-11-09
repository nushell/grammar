#!/usr/bin/env python3
"""Snippets for the corpus: every keyword statement with a help flag or a flag
value written into each slot after its keyword, and the fixed-signature keyword commands with
switch values, short-flag batches and help followed by more arguments.
usage: gen_sweep.py OUT.json"""
import json, sys

STATEMENTS = [
    'def f [] {}', 'def --env f [] {}', 'def --wrapped f [...r] {}', 'export def f [] {}',
    'extern e []', 'extern --strict e [--a]', 'export extern e []',
    'alias a = ls', 'export alias a = ls', 'type t = int', 'export type t = int',
    'use std/assert', 'export use std/assert', 'module m {}', 'export module m {}',
    'const c = 1', 'export const c = 1', 'let x = 1', 'mut x = 1',
    'for x in [1] {}', 'while true {}', 'loop {}', 'if true { 1 }', 'if true { 1 } else { 2 }',
    'match 1 { _ => 2 }', 'try {}', 'try {} catch {}', 'return 1', 'break', 'continue',
    'source null', 'source-env null', 'hide ls', 'hide-env FOO', 'export-env {}', 'export',
    'overlay new o', 'overlay use o', 'overlay use o as p', 'overlay hide o', 'overlay list',
    'plugin use query', 'attr example "a" {1}', 'attr category x', 'attr search-terms a b',
    'unlet $x', 'where true', 'do {}',
]
FLAGS = ['--help', '-h', '--help=false', '--help=true', '--help=1', '--help=$x', '--help=(true)',
         '--help=', '-h=x', '-hx', '-xh', '--help={a: 1}', '--', '--bogus', '--bogus=1']
SIGNATURE_STATEMENTS = {
    'overlay new o': ['--reload', '-r'],
    'overlay use o': ['--prefix', '--reload', '-p', '-r'],
    'overlay hide o': ['--keep-custom', '-k'],
    'run x': ['--full-reparse'],
}
SWITCH_FORMS = ['{f}=false', '{f}=true', '{f}=1', '{f}=$x', '{f}=(1 == 1)', '{f}=', '{f}={{a: 1}}']
BATCHES = ['-pr', '-rp', '-hp', '-ph', '-pz', '-zp', '-hz', '-ek', '-ke [A]', '-ke', '-kh', '-ekh', '-p=1']

def slots(statement):
    words = statement.split(' ')
    head = 2 if words[0] in ('export', 'overlay', 'plugin', 'attr') and len(words) > 1 else 1
    for i in range(head, len(words) + 1):
        yield words[:i], words[i:]

out = []
for statement in STATEMENTS:
    for before, after in slots(statement):
        for flag in FLAGS:
            out.append(' '.join(before + [flag] + after))
        # help followed by an extra word, and help twice
        out.append(' '.join(before + ['--help', 'extra'] + after))
        out.append(' '.join(before + ['--help', '--bogus'] + after))
for statement, flags in SIGNATURE_STATEMENTS.items():
    for before, after in slots(statement):
        for flag in flags:
            if flag.startswith('--'):
                for form in SWITCH_FORMS:
                    out.append(' '.join(before + [form.format(f=flag)] + after))
        for batch in BATCHES:
            out.append(' '.join(before + [batch] + after))
for extra in ['hide --help a b c', 'source --help a b', 'overlay use --help a b', 'overlay use --help a as',
              'overlay use --help a as b c', 'overlay hide --help --keep-env', 'plugin use --help --plugin-config',
              'overlay hide --keep-env=[A] o', 'overlay hide --keep-env=1 o', 'hide --help foo null',
              'overlay use --help -1 x', 'attr example --help a', 'attr example --help']:
    out.append(extra)
seen, snippets = set(), []
for text in out:
    if text not in seen:
        seen.add(text)
        snippets.append({"origin": "flag-sweep", "snippet": text})
json.dump(snippets, open(sys.argv[1], 'w'))
print(len(snippets))
