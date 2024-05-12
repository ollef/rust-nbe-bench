use rixty::{
    domain_rc,
    index::{Index, Level},
    syntax::{self, TermRef},
};

fn to_nat<'a>(term: TermRef<'a>) -> usize {
    match term {
        syntax::Term::Lambda(syntax::Term::Lambda(body)) => {
            let mut result = 0;
            let mut body = *body;
            while let syntax::Term::Application(syntax::Term::Variable(Index(1)), arg) = body {
                result += 1;
                body = arg;
            }
            match body {
                syntax::Term::Variable(Index(0)) => result,
                _ => panic!("Not a nat"),
            }
        }
        _ => panic!("Not a nat"),
    }
}

fn main() {
    let b = syntax::Builder::new();
    let v0 = b.v(0);
    let v1 = b.v(1);
    let v2 = b.v(2);
    let v3 = b.v(3);
    let n2 = b.l(b.l(b.apps(v1, &[v1, v0])));
    let n5 = b.l(b.l(b.apps(v1, &[v1, v1, v1, v1, v0])));
    let mul = b.l(b.l(b.l(b.l(b.apps(v3, &[b.application(v2, v1), v0])))));
    let suc = |n| b.l(b.l(b.application(v1, b.apps(n, &[v1, v0]))));

    let n10 = b.apps(mul, &[n2, n5]);
    let n10b = b.apps(mul, &[n5, n2]);
    let n20 = b.apps(mul, &[n2, n10]);
    let n20b = b.apps(mul, &[n2, n10b]);
    let n21 = suc(n20);
    let n21b = suc(n20b);
    let n22 = suc(n21);
    let n22b = suc(n21b);
    let n100 = b.apps(mul, &[n10, n10]);
    let n100b = b.apps(mul, &[n10b, n10b]);
    let n10k = b.apps(mul, &[n100, n100]);
    let n10kb = b.apps(mul, &[n100b, n100b]);
    let n100k = b.apps(mul, &[n10k, n10]);
    let n100kb = b.apps(mul, &[n10kb, n10b]);
    let n1M = b.apps(mul, &[n10k, n100]);
    let n1Mb = b.apps(mul, &[n10kb, n100b]);
    let n5M = b.apps(mul, &[n1M, n5]);
    let n5Mb = b.apps(mul, &[n1Mb, n5]);
    let n10M = b.apps(mul, &[n1M, n10]);
    let n10Mb = b.apps(mul, &[n1Mb, n10b]);
    let n20M = b.apps(mul, &[n10M, n2]);
    let n20Mb = b.apps(mul, &[n10Mb, n2]);
    let leaf = b.l(b.l(v1));
    let node = b.l(b.l(b.l(b.l(b.apps(v0, &[b.apps(v3, &[v1, v0]), b.apps(v2, &[v1, v0])])))));
    let full_tree = b.l(b.apps(v0, &[b.l(b.apps(node, &[v0, v0])), leaf]));
    let tree_2M = b.application(full_tree, n20);
    let tree_4M = b.application(full_tree, n21);
    let tree_8M = b.application(full_tree, n22);

    let result_builder = syntax::Builder::new();
    tree_2M
        .evaluate_rc(&mut domain_rc::Environment::new())
        .quote(Level(0), &result_builder);
}
