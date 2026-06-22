## Variables

Variables can have any name which does not start with a number.
Variables are mutable, and always passed by value to functions.

### Declaration

Variables need to be declared to be used. Declaration and definition are always combined, so that undefined variables do not exist.
```zarathustra
{{#include ../../../zarathustra_cli/examples/book/declaration.zok}}
```

### Mutability

Variables are immutable by default. In order to declare a mutable variable, the `mut` keyword is used.
```zarathustra
{{#include ../../../zarathustra_cli/examples/book/mutable.zok}}
```

### Shadowing

Shadowing is allowed.
```zarathustra
{{#include ../../../zarathustra_cli/examples/book/shadowing.zok}}
```

### Scope

#### Function

Functions have their own scope
```zarathustra
{{#include ../../../zarathustra_cli/examples/book/function_scope.zok}}
```

#### For-loop
For-loops have their own scope
```zarathustra
{{#include ../../../zarathustra_cli/examples/book/for_scope.zok}}
```