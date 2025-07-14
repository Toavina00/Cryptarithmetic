use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

type Value = i32;
type Domain = HashSet<Value>;
type Variables = HashMap<String, Domain>;

enum Constraint {
    Binary((String, String, Rc<dyn Fn(Value, Value) -> bool>)),
    Unary((String, Rc<dyn Fn(Value) -> bool>)),
}

fn consistency(variables: &mut Variables, arcs: &Vec<Constraint>) {
    for arc in arcs.iter() {
        match arc {
            Constraint::Binary((x0, x1, a)) => {
                let mut consistent = HashSet::new();

                if let Some(x0_values) = variables.get(x0) {
                    if let Some(x1_values) = variables.get(x1) {
                        consistent = x0_values
                            .iter()
                            .filter(|x| x1_values.iter().any(|y| a(**x, *y)))
                            .copied()
                            .collect();
                    }
                }

                if let Some(x0_values) = variables.get_mut(x0) {
                    *x0_values = consistent;
                }
            }

            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get_mut(x) {
                    x_values.retain(|v| a(*v));
                }
            }
        }
    }
}

fn verify(variables: &Variables, constraints: &Vec<Constraint>) -> bool {
    for constraint in constraints.iter() {
        match constraint {
            Constraint::Binary((x0, x1, a)) => {
                if let Some(x0_values) = variables.get(x0) {
                    if let Some(x1_values) = variables.get(x1) {
                        if x0_values
                            .iter()
                            .any(|x| x1_values.iter().all(|y| !a(*x, *y)))
                        {
                            return false;
                        }
                    }
                }
            }

            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get(x) {
                    if x_values.iter().any(|v| !a(*v)) {
                        return false;
                    }
                }
            }
        }
    }

    true
}

fn build_arcs(constraints: &Vec<Constraint>) -> Vec<Constraint> {
    let mut arcs: Vec<Constraint> = Vec::new();

    for constraint in constraints.iter() {
        match constraint {
            Constraint::Binary((x0, x1, f)) => {
                let func = f.clone();
                arcs.push(Constraint::Binary((x0.clone(), x1.clone(), f.clone())));
                arcs.push(Constraint::Binary((
                    x1.clone(),
                    x0.clone(),
                    Rc::new(move |b, a| func(a, b)),
                )));
            }

            Constraint::Unary((x, f)) => {
                arcs.push(Constraint::Unary((x.clone(), f.clone())));
            }
        }
    }

    arcs
}

fn main() {
    let mut t0 = String::from("ISA");
    let mut t1 = String::from("ROA");
    let mut t2 = String::from("TELO");

    let m_len = max(t0.len(), max(t1.len(), t2.len()));

    t0 = format!("{:#>w$}", t0, w = m_len);
    t1 = format!("{:#>w$}", t1, w = m_len);
    t2 = format!("{:#>w$}", t2, w = m_len);

    let mut constraints: Vec<Constraint> = Vec::new();
    let mut variables: Variables = Variables::new();

    variables.insert(String::from("#"), HashSet::from_iter([0]));

    let mut iter_t0 = t0.chars().rev();
    let mut iter_t1 = t1.chars().rev();
    let mut iter_t2 = t2.chars().rev();

    loop {
        let n0 = iter_t0.next();
        let n1 = iter_t1.next();
        let n2 = iter_t2.next();

        if n0.is_none() && n1.is_none() && n2.is_none() {
            break;
        }
    }

    let arcs: Vec<Constraint> = build_arcs(&constraints);

    loop {
        consistency(&mut variables, &arcs);
        if verify(&variables, &constraints) {
            break;
        }
    }

    println!("{:?}", variables);
}
