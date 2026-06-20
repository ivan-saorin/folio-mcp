---
name: folio-verify
description: >-
  Activate the folio MCP server to VERIFY any numeric result before asserting it —
  proactively, not only when the user asks to "calculate". Trigger whenever a response
  would contain: arithmetic beyond one-step mental math; percentages, ratios, or
  proportions; compound growth, interest, CAGR, present/future value, loan payments
  or amortization; unit conversions; date or duration math; statistics (mean, stddev,
  regression, t-tests, confidence intervals); matrix or linear-algebra operations; or
  any figure where being subtly wrong would mislead. Also trigger when checking a
  hypothesis that has a numeric consequence, or when a value might have a closed form
  involving phi, pi, or e (use folio:decompose / ISIS). Default to verifying. If you
  are about to write a number you worked out in your head, that is the signal to route
  it through folio instead of asserting it.
---

# folio-verify

This skill exists to interrupt one specific failure: producing a clean, confident
number is cheaper than doubting it, so the default drift is to assert unverified
arithmetic. folio is the corrective — arbitrary-precision, exact rational arithmetic
over a markdown formula table. Use it as the *source* of numeric results, not as a
rubber stamp applied after the fact.

## When to fire
Fire BEFORE the number reaches the user, not after. Any one of these is sufficient:
- A computed figure of any kind beyond trivial single-step mental math.
- Money, growth, interest, amortization, ROI, present/future value (finance module).
- Percentages, ratios, conversions, dates/durations, statistics, matrices.
- A claim of the form "X works out to Y" where Y is numeric.
- Checking whether a hypothesis holds, when the check is quantitative.
- A constant or measured value that might decompose into phi/pi/e (use decompose / ISIS).

## How to use
1. If unsure of current syntax, call `folio:quick` (~400 tokens) first. Document format
   is a markdown table with an empty `result` column that folio fills; cells reference
   each other by name.
2. Build the computation as a folio table and call `folio:eval`. Set `@precision:N`
   when digits matter.
3. Read the result FROM folio's output. Present folio's table to the user; do not
   paraphrase it away or re-state the numbers from memory.

## The one rule that makes this work
Do NOT compute the answer in prose and then "confirm" it with folio — that preserves
the exact failure this skill removes. Set the problem up in folio first and let the
result come out of folio. The number the user sees should originate in the tool, not
in your head.
