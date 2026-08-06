---
name: Bug report
about: Create a report to help us improve Astra
title: ''
labels: bug
assignees: ''
---

## Describe the bug
A clear and concise description of what the bug is.

## To reproduce
Steps to reproduce the behaviour; include the smallest `.zara` circuit if relevant:

```
def main(field public sum, field private x, field private y) -> field {
    assert(x + y == sum);
    return 1;
}
```

## Expected behaviour
What you expected to happen.

## Actual behaviour
What happened instead. Include the full error/output.

## Environment
- OS:
- Rust version (`rustc --version`):
- Cargo version (`cargo --version`):
- Commit (`git rev-parse HEAD`):

## Additional context
- Does it reproduce with `cargo build --workspace` clean?
- Does `cargo audit --deny high .` report related findings?