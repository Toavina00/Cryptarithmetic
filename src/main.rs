#![allow(dead_code)]

use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::usize;

//type Value = i32;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
enum Value {
    Scalar(i32),
    Vector([i32; 27]),
}

impl Value {
    fn get_value(&self) -> Result<i32, ()> {
        if let Value::Scalar(x) = *self {
            return Ok(x)
        }
        Err(())
    }
}

type Domain = HashSet<Value>;
type Variables = HashMap<String, Domain>;

#[derive(Clone)]
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

fn filter_domain(variables: &mut Variables, constraints: &Vec<Constraint>) {
    let arcs: Vec<Constraint> = build_arcs(&constraints);

    loop {
        consistency(variables, &arcs);
        if verify(variables, constraints) {
            break;
        }
    }
}

fn main() {
    let mut input0 = "ISA".to_string();
    let mut input1 = "ROA".to_string();
    let mut output = "TELO".to_string();

    let mut variables: Variables = Variables::new();
    let mut constraints: Vec<Constraint> = Vec::new();

    // Initialize Variables
    input0
        .to_uppercase()
        .chars()
        .collect::<HashSet<char>>()
        .union(
            &input1
                .to_uppercase()
                .chars()
                .collect::<HashSet<char>>()
                .union(&output.to_uppercase().chars().collect::<HashSet<char>>())
                .copied()
                .collect::<HashSet<char>>(),
        )
        .copied()
        .for_each(|s| {
            variables.insert(
                s.to_string(),
                HashSet::from_iter((0..=9).map(|v| Value::Scalar(v))),
            );
        });

    // Insert Special padding character '#'
    let width = max(input0.len(), max(input1.len(), output.len()));

    input0 = format!("{:#>w$}", input0, w = width);
    input1 = format!("{:#>w$}", input1, w = width);
    output = format!("{:#>w$}", output, w = width);

    variables.insert("#".to_string(), HashSet::from_iter([Value::Scalar(0)]));

    constraints.push(Constraint::Unary((
        "#".to_string(),
        Rc::new(|v| {
            if let Value::Scalar(x) = v {
                return x == 0;
            }
            false
        }),
    )));

    // Initialize Constraints
    for i in 0..width {
        let c0 = input0.chars().nth(i).unwrap();
        let c1 = input1.chars().nth(i).unwrap();
        let c2 = output.chars().nth(i).unwrap();

        let (mut i0, mut i1, mut i2) = (26, 26, 26);

        for (c, j) in [c0, c1, c2].iter().zip([&mut i0, &mut i1, &mut i2]) {
            if c.to_string() != "#" {
                *j = (*c as usize) - ('A' as usize);
            }
        }

        // Create Hidden Variables
        let mut hidden_variable = Domain::new();

        for va in variables.get(&c0.to_string()).unwrap().iter() {
            for vb in variables.get(&c1.to_string()).unwrap().iter() {
                for vc in variables.get(&c2.to_string()).unwrap().iter() {
                    if let Value::Scalar(a) = va {
                        if let Value::Scalar(b) = vb {
                            if let Value::Scalar(c) = vc {
                                let mut val = [0; 27];
                                val[i0] = *a;
                                val[i1] = *b;
                                val[i2] = *c;
                                hidden_variable.insert(Value::Vector(val));
                            }
                        }
                    }
                }
            }
        }

        let h_name = format!("HIDDEN_{}", i);

        variables.insert(
            h_name.to_string(),
            hidden_variable
        );

        // Unary Constraint Hidden Variables
        constraints.push(Constraint::Unary((
            h_name.clone(),
            Rc::new(move |v| {
                if let Value::Vector(u) = v {
                    let mut output = ((u[i0] + u[i1]) % 10) == u[i2];
                    if i < width - 1 {
                        output = output || ((u[i0] + u[i1] + 1) % 10) == u[i2];
                    }
                    return output;
                }
                false
            }),
        )));

        // Binary Constraint Hidden Variables <-> Original Variables
        for (c, j) in [c0, c1, c2].iter().zip([i0, i1, i2]) {
            constraints.push(Constraint::Binary((
                c.to_string(),
                h_name.clone(),
                Rc::new(move |a, b| {
                    if let Value::Scalar(x) = a {
                        if let Value::Vector(u) = b {
                            return x == u[j];
                        }
                    }
                    false
                }),
            )));
        }
    }

    // Filter the domain
    filter_domain(&mut variables, &constraints);

    // Remove Hidden Variables
    variables.retain(|k, _| !k.contains("HIDDEN"));

    // Print Results
    println!(
        "{:?}",
        variables
            .iter()
            .map(|(k, v)| (k, v.iter().filter_map(|x| x.get_value().ok()).collect::<Vec<i32>>()))
            .collect::<Vec<(&String, Vec<i32>)>>()
    );
}
