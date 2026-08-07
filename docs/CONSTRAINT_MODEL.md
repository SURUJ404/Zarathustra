# Astra Constraint Ground Truth

This is the **mathematical ground truth** for the Zara → R1CS lowering. It is the
reference against which fuzzing, the compiler, and any future backend are judged.
Every construct maps to a deterministic witness layout and a deterministic set of
constraints **over the BLS12-381 scalar field** `F_p`.

Stability rule: *the compiler emits constraints lazily in exact postfix
evaluation order.* No construct is reordered, simplified, or collapsed by the
compiler — the ground truth below is the ordering the compiler must produce.

---

## 0. Quick reference (every construct)

For each construct: expected mathematical semantics → expected constraints →
expected witness.

| # | Construct | Semantics | Constraints | Witness |
|---|-----------|-----------|-------------|---------|
| 1 | param `field p` (public) | `p = pub[k]` | 0 | public var |
| 1 | param `field private p` | `p = priv[m]` | 0 | private var |
| 2 | variable `a` | `a` | 0 | LC `[(idx(a),1)]` |
| 3 | literal `5` | `5` | 0 | LC `[(0,5)]` (on `~one`) |
| 4 | `a + b` | `a + b` | 0 (merge) | LC union, coeffs summed |
| 5 | `a − b` | `a − b` | 0 (merge) | LC `lc(a) ∪ −lc(b)` |
| 6 | `a * b` | `a·b` | **1 row** `A·B=C` | new private `~mult_k = eval(a)·eval(b)` |
| 7 | ` | `l = r` |2.*` to `0.*` | 1 row `1·(l−r)=0` |
| 8 | `field t = <init>` | `t = init` | init's rows | 1 new private var `w[idx]=eval(init)` |
| 9 | `if (c){body}` | cond discarded, body inlined | `0 + body rows` | no cond var; body normal |
| 10 | `return e` | no-op | 0 | none |

> Note (row 7): `assert(l == r)` emits **exactly one** row of shape
> `1 · (left − right) = 0`.

---

## 1. Witness index convention

All rows are indexed over the **witness vector** `w[0..n]`.

| order | name            | public/private | value                      |
|-------|-----------------|----------------|----------------------------|
| 0     | `~one`          | public (implicit) | `1`                        |
| 1..   | `main` params   | per declaration | input values               |
| then  | `Stmt::Declare` | private        | init value or `0`          |
| then  | `~mult_<k>`     | private        | `l * r` (in postfix order) |

- `num_public` starts at **1** (the implicit `~one` at index 0).
- `num_private` counts every witness index past the public block.
- Params are allocated in program order. A param marked `private` becomes
  private; all others are public.

---

## 2. Complexity model

Each construct's worst-case cost: (a) new witness entries, (b) new R1CS rows,
(c) arithmetic cost. `L = len(linear combination)`.

| construct       | new vars | new rows | cost |
|-----------------|----------|----------|------|
| `~one` (implicit) | 1 | 0 | `O(1)` |
| param           | 1 | 0 | `O(1)` |
| `var`           | 0 | 0 | `O(1)` |
| literal `c`     | 0 | 0 | `O(1)` |
| `a + b`         | 0 | 0 | `O(L_a + L_b)` |
| `a − b`         | 0 | 0 | `O(L_a + L_b)` |
| `a * b`         | 1 | 1 | `O(L_a + L_b)` |
| `assert(l==r)`  | 0 | 1 | `O(L_l + L_r)` |
| `declare`       | 1 | 0 | `O(init)` |
| `if`            | 0 | `+body` rows | `O(body)` (always) |
| `return`        | 0 | 0 | `O(0)` |

Total compile cost for a straight-line circuit of `m` constraints over at most
`n` active vars is **`O(m·n)`**; a linear-definition (one LC term per var) keeps
it at `O(m)`.

---

## 3. Per-construct encoding (the oracle)

### 3.1 Program / params
- `def main(params) -> ret { body }` — params become vars in order; `-> ret`
  is discarded; only `main` is legal.
- public param: `w[idx] = pub[k]`; private param: `w[idx] = priv[m]`.
- No rows.

### 3.2 Literal & variable
- `compile_expr(Number)` → `[(0, val)]` (the literal rides on `~one`).
- `compile_expr(Variable)` → `[(idx, 1)]`.

### 3.3 Addition / subtraction
No row. Returns the merged LC:
- `a + b`: terms of `a` and `b` summed at matching indices, zero terms dropped.
- `a − b`: terms of `a` and `−b`.

### 3.4 Multiplication (gate) — `a * b`
Emits **exactly one row** `A·B = C`:
```
new private var idx = next var (named ~mult_<k>, k = #rows so far)
witness[idx]      = eval(a) · eval(b)
row.k:  A = lc(a),  B = lc(b),  C = [(idx, 1)]
```
Ground truth witness check: `w[idx] == eval(a) * eval(b)`.

### 3.5 Assert equal — `l == r`
Emits exactly one row `1 · (l − r) = 0`:
```
A = [(0, 1)]        // ~one
B = lc(l) − lc(r)
C = []              // empty
```
Ground truth: `w[l] − w[r] = 0`.

### 3.6 Declare — `field t = <init>`
Fresh private var with the init value (or `0` if unspecified):
```
idx = next var
witness[idx] = eval(init)   // 0 if no init
```

### 3.7 if
Condition evaluated and its LC **discarded**; body always lowered.
Ground truth: `num_constraints(if)` equals body constraints; the condition adds
no row.

### 3.8 return — no-op.
Emits nothing.

---

## 4. Collision resistance / binding

Every **private** `~mult_<k>` and `declare` variable is uniquely indexed. The
R1CS encoding `a · b = c` binds the outputs of multiplicative gates, and the
`1 · (x − y) = 0` shape binds equality. Two different witness assignments that
both satisfy every row yield the same public result if and only if they agree on
every **bound** variable.

The index assignment is deterministic: given the same program string and the
same input values, the witness vector and the `(A,B,C)` triples are
bit-for-bit reproducible across compiles — this is the invariance fuzzing
checks. Distinct program semantics (different DAGs) must produce distinct
constraint/witness artifacts; the fuzzer's collision gate asserts this.

---

## 5. Worked example (chain) — `add`

```zara
def main(field public c, field private a, field private b) -> field {
    field t = a + b;   // Declare(0 rows)
    assert(t == c);    // Constrain(1 row)
    return 1;          // no-op
}
```

| Zara | semantic | constraints | witness |
|------|----------|-------------|---------|
| `~one` | implicit | — | `w[0]=1` |
| `c` | pub input | — | `w[1]=7` |
| `a` | private input | — | `w[2]=3` |
| `b` | private input | — | `w[3]=4` |
| `t = a + b` | `a + b` | `0` | `w[4]=7` |
| `assert(t == c)` | `(a+b) − c = 0` | row 0: `A=[(0,1)] B=[(4,1),(1,−1)] C=[]` | — |
| `return 1` | no-op | `0` | — |

Totals: `num_variables = 5`, `num_constraints = 1`, `num_public = 2`,
`num_private = 3`.

---

## 6. Example B — one multiplicative gate `c = a·b`

```c
def main(field public c, field private a, field private b) -> field {
    field t = a * b;   // Declare -> ~mult_0 row
    assert(t == c);     // Constrain row
    return 1;           // no-op
}
```

Witness: `[~one, c, a, b, t, ~mult_0]` → `num_variables = 6`, `num_private = 4`,
`num_constraints = 2`.

```
row 0: A=[(2,1)] B=[(3,1)] C=[(5,1)]    // a·b = ~mult_0
row 1: A=[(0,1)] B=[(4,1),(1,−1)] C=[]   // t − c = 0
```

Any change to this layout (index order, term ordering, count) is a
**ground truth violation** — fuzzers and reviewers check exactly these
invariants.