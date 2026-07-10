use crate::Finding;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Expr, ExprBinary, ExprCall, ExprMethodCall, ItemFn, Macro, Stmt,
};
use quote::ToTokens;

pub fn analyze(code: &str, file: &str) -> Vec<Finding> {
    let syntax = match syn::parse_file(code) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut collector = FindingCollector {
        findings: Vec::new(),
        file: file.to_string(),
        lines: code.lines().map(|l| l.to_string()).collect(),
    };

    collector.visit_file(&syntax);
    collector.findings
}

struct FindingCollector {
    findings: Vec<Finding>,
    file: String,
    lines: Vec<String>,
}

impl FindingCollector {
    fn snippet_at(&self, line: usize) -> String {
        self.lines
            .get(line.saturating_sub(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }

    fn push(&mut self, severity: &str, category: &str, title: &str, line: usize, desc: &str, fix: &str) {
        self.findings.push(Finding {
            severity: severity.to_string(),
            category: category.to_string(),
            title: title.to_string(),
            file: self.file.clone(),
            line: Some(line),
            description: desc.to_string(),
            snippet: self.snippet_at(line),
            fix: fix.to_string(),
        });
    }

    fn line_of(&self, node: &impl Spanned) -> usize {
        node.span().start().line
    }
}

impl<'ast> Visit<'ast> for FindingCollector {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method_name = node.method.to_token_stream().to_string();

        match method_name.as_str() {
            "unwrap" => {
                let loc = self.line_of(&node.method);
                self.push(
                    "HIGH",
                    "Panic Site",
                    "Unwrap Without Error Handling",
                    loc,
                    "Calling .unwrap() will panic if the Result or Option is None/Err. Prefer pattern matching, 'if let', '?', or expect().",
                    "Replace .unwrap() with match, '?', or .expect(\"message\") for safe error handling.",
                );
            }
            "expect" => {
                let loc = self.line_of(&node.method);
                self.push(
                    "MEDIUM",
                    "Panic Site",
                    "Assertive Expect Call",
                    loc,
                    ".expect() panics with a custom message when the value is None/Err. Ensure the invariant is truly impossible to violate.",
                    "Consider using '?' or match instead, or keep .expect() only when the failure case is truly unreachable.",
                );
            }
            _ => {}
        }

        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(ref path_expr) = *node.func {
            let name = path_expr
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string());

            if let Some(n) = name {
                let loc = self.line_of(&node.func);
                match n.as_str() {
                    "Box::leak" | "Box::into_raw" | "Vec::leak" => {
                        self.push(
                            "HIGH",
                            "Memory Leak",
                            &format!("Potential Memory Leak from {}", n),
                            loc,
                            &format!("{} deliberately leaks memory. Ensure this is intentional and the returned pointer lifetime is properly managed.", n),
                            "Ensure the leaked memory is freed at the correct point with drop() or by reconstructing ownership.",
                        );
                    }
                    _ => {}
                }
            }
        }

        syn::visit::visit_expr_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let name = node.path.segments.last().map(|s| s.ident.to_string());

        if let Some(n) = name {
            let loc = self.line_of(&node.path);
            match n.as_str() {
                "panic" => {
                    self.push(
                        "HIGH",
                        "Panic Site",
                        "Explicit Panic Invocation",
                        loc,
                        "panic!() causes abrupt program termination. Prefer returning Result for recoverable errors.",
                        "Replace panic!() with returning a Result<_, E> or using compile-time guarantees.",
                    );
                }
                "unreachable" => {
                    self.push(
                        "MEDIUM",
                        "Panic Site",
                        "Unreachable Code Marker",
                        loc,
                        "unreachable!() will panic if reached at runtime. Ensure exhaustive pattern coverage.",
                        "Verify all enum variants are matched, or replace with a fallback case that returns an error.",
                    );
                }
                "unimplemented" => {
                    self.push(
                        "MEDIUM",
                        "Panic Site",
                        "Unimplemented Code Path",
                        loc,
                        "unimplemented!() panics if this code path is ever hit during execution.",
                        "Implement the missing functionality or return a proper error instead.",
                    );
                }
                "todo" => {
                    self.push(
                        "LOW",
                        "Panic Site",
                        "Todo Marker",
                        loc,
                        "todo!() will panic if this code path is reached.",
                        "Implement the remaining logic before production use.",
                    );
                }
                "assert" | "debug_assert" => {
                    self.push(
                        "LOW",
                        "Panic Site",
                        &format!("{} Used", n),
                        loc,
                        &format!("{} panics if the condition is false. Ensure invariants are truly guaranteed at runtime.", n),
                        &format!("Use Result for expected failure conditions, or keep {} only for true invariant checks.", n),
                    );
                }
                _ => {}
            }
        }

        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        let loc = self.line_of(node);
        let block_str = node.block.to_token_stream().to_string();
        let has_pointer = block_str.contains("as *const") || block_str.contains("as *mut");

        let severity = if has_pointer { "CRITICAL" } else { "HIGH" };
        let mut desc = "Unsafe block bypasses Rust's safety guarantees.".to_string();
        if has_pointer {
            desc.push_str(" Raw pointers are created inside — the compiler cannot verify memory safety.");
        }
        let title = if has_pointer {
            "Unsafe Block with Raw Pointers"
        } else {
            "Unsafe Code Block"
        };

        self.push(
            severity,
            "Unsafe Usage",
            title,
            loc,
            &desc,
            "Minimize unsafe code. Encapsulate it in small, audited functions with safe abstractions.",
        );

        syn::visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        if node.sig.unsafety.is_some() {
            let loc = self.line_of(&node.sig);
            self.push(
                "HIGH",
                "Unsafe Usage",
                "Unsafe Function Declaration",
                loc,
                &format!("Function '{}' is declared unsafe. All callers must verify preconditions.", node.sig.ident),
                "Prefer safe abstractions. If unsafe is required, document safety preconditions in doc comments.",
            );
        }

        let ret_str = node.sig.output.to_token_stream().to_string();
        if ret_str.contains("*const ") || ret_str.contains("*mut ") {
            let loc = self.line_of(&node.sig);
            self.push(
                "HIGH",
                "Unsafe Usage",
                "Function Returns Raw Pointer",
                loc,
                &format!("Function '{}' returns a raw pointer. Callers must ensure proper lifetime management.", node.sig.ident),
                "Prefer references with explicit lifetimes, or encapsulate in a safe wrapper type.",
            );
        }

        syn::visit::visit_item_fn(self, node);
    }

    fn visit_expr_binary(&mut self, node: &'ast ExprBinary) {
        use syn::BinOp::*;
        let is_arithmetic = matches!(node.op, Add(_) | Sub(_) | Mul(_) | Div(_) | Rem(_));

        if is_arithmetic {
            let loc = self.line_of(node);
            let lhs_line = self.lines.get(loc.saturating_sub(2)).map(|s| s.trim()).unwrap_or("");

            let is_safe_arithmetic = lhs_line.contains("wrapping_")
                || lhs_line.contains("checked_")
                || lhs_line.contains("overflowing_")
                || lhs_line.contains("saturating_");

            if !is_safe_arithmetic {
                let lhs_s = node.left.to_token_stream().to_string();
                let rhs_s = node.right.to_token_stream().to_string();
                if lhs_s.len() < 50 && rhs_s.len() < 50 {
                    let op_c = match node.op {
                        Add(_) => '+',
                        Sub(_) => '-',
                        Mul(_) => '*',
                        Div(_) => '/',
                        Rem(_) => '%',
                        _ => '?',
                    };
                    self.push(
                        "LOW",
                        "Unchecked Arithmetic",
                        &format!("Potential Integer Overflow ({})", op_c),
                        loc,
                        &format!("Arithmetic operation '{} {} {}' may overflow in debug mode or wrap in release mode.",
                            lhs_s.trim_start_matches('"').trim_end_matches('"'),
                            op_c,
                            rhs_s.trim_start_matches('"').trim_end_matches('"'),
                        ),
                        "Consider checked_add(), wrapping_add(), saturating_add(), or ensure inputs are bounded.",
                    );
                }
            }
        }

        syn::visit::visit_expr_binary(self, node);
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if let Stmt::Local(local) = node {
            let pat_str = local.pat.to_token_stream().to_string();
            if let Some(init) = &local.init {
                let init_str = init.expr.to_token_stream().to_string();
                let loc = self.line_of(&local.pat);

                if pat_str.contains("Arc") && init_str.contains("Mutex") {
                    self.push(
                        "INFO",
                        "Concurrency",
                        "Arc<Mutex> Detected",
                        loc,
                        "Arc<Mutex<T>> enables shared mutable state across threads. Ensure locking discipline is correct.",
                        "Document which locks protect which data. Consider replacing with atomics or message passing where possible.",
                    );
                }
                if pat_str.contains("Arc") && init_str.contains("RwLock") {
                    self.push(
                        "INFO",
                        "Concurrency",
                        "Arc<RwLock> Detected",
                        loc,
                        "Arc<RwLock<T>> enables concurrent reads with exclusive writes. Watch for write starvation and deadlocks.",
                        "Use RwLock when read frequency >> write frequency. Otherwise prefer Mutex.",
                    );
                }
            }
        }

        syn::visit::visit_stmt(self, node);
    }
}
