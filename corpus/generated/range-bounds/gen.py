#!/usr/bin/env python3
"""Range items whose bound is a group or a raw string with and without text after it, as a
statement, as an argument and as a value. usage: gen.py OUT.json"""
import json, sys
BOUNDS = ['(1)', '(1)x', '(ls).0', '(1).a', '(1).a?', '(1).0x', '{a: 1}.a', '{a:1}print', '{a:1}', '{}', '{||}',
          '{|| 1}x', '{ls}', "r#'a'#", "r#'a'#b", "r##'a'##", "r#'a'#.0", '$x', '$x.a', '-(1)', '-(1)x',
          '1', '[1]', '"a"', '"a"x', '(1)(2)', '(1).(2)', '{a: 1}.a.b', "{a: 1}.'b c'", '(1)!']
FORMS = ['{b}..5', '5..{b}', '..{b}', '{b}..', '1..{b}..5', '{b}..<5']
CONTEXTS = ['{r}', '{r}print', 'echo {r}', 'let r = {r}', 'let x = 1; {r}']
out = []
for b in BOUNDS:
    for f in FORMS:
        r = f.format(b=b)
        for c in CONTEXTS:
            out.append({"origin": "range-bounds", "snippet": c.format(r=r)})
json.dump(out, open(sys.argv[1], 'w'))
print(len(out))
