use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

mod ac;

use ac::*;


fn main() {
    let mut input0 = "ISA".to_string();
    let mut input1 = "ROA".to_string();
    let mut output = "TELO".to_string();

    let mut variables: Variables<i32> = Variables::new();
    let mut constraints: Constraints<i32> = Vec::new();

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
            variables.insert(s.to_string(), Vec::from_iter(0..=9));
        });

    // Inequalities Constraints
    for x in variables.variable.keys() {
        for y in variables.variable.keys() {
            if *x != *y {
                constraints.push(Constraint::Binary((
                    x.clone(),
                    y.clone(),
                    Rc::new(|a, b| {
                        if let (VariableType::Value(x), VariableType::Value(y)) = (a, b) {
                            return *x != *y;
                        }
                        false
                    }),
                )));
            }
        }
    }

    // Insert Special padding character '#'
    let width = max(input0.len(), max(input1.len(), output.len()));

    input0 = format!("{:#>w$}", input0, w = width);
    input1 = format!("{:#>w$}", input1, w = width);
    output = format!("{:#>w$}", output, w = width);

    variables.insert("#".to_string(), vec![0]);

    constraints.push(Constraint::Unary((
        "#".to_string(),
        Rc::new(|v| {
            if let VariableType::Value(x) = v {
                return *x == 0;
            }
            false
        }),
    )));

    // Add Carry variables
    for i in 0..width {
        let carry = format!("CARRY_{}", i);
        variables.insert(carry, vec![0, 1]);
    }

    constraints.push(Constraint::Unary((
        format!("CARRY_{}", width - 1),
        Rc::new(|v| {
            if let VariableType::Value(x) = v {
                return *x == 0;
            }
            false
        }),
    )));


    // Initialize Constraints for each column
    for i in 0..width {
        let c0 = input0.chars().nth(i).unwrap();
        let c1 = input1.chars().nth(i).unwrap();
        let c2 = output.chars().nth(i).unwrap();
        let carry_in = format!("CARRY_{}", i);
        let carry_out= if i == 0 {"#".to_string()} else {format!("CARRY_{}", i-1)};

        // Create Hidden Variables
        let mut hidden_variable = Vec::new();
        for va in variables.get(&c0.to_string()).unwrap().iter() {
            for vb in variables.get(&c1.to_string()).unwrap().iter() {
                for vc in variables.get(&c2.to_string()).unwrap().iter() {
                    for vci in variables.get(&carry_in).unwrap().iter() {
                        for vco in variables.get(&carry_out).unwrap().iter() {
                            if let (
                                VariableType::Value(a),
                                VariableType::Value(b),
                                VariableType::Value(c),
                                VariableType::Value(ci),
                                VariableType::Value(co),
                            ) = (va, vb, vc, vci, vco)
                            {
                                let mut h_map = HashMap::new();
                                h_map.insert(c0.to_string(), **a);
                                h_map.insert(c1.to_string(), **b);
                                h_map.insert(c2.to_string(), **c);
                                h_map.insert(carry_in.clone(), **ci);
                                h_map.insert(carry_out.clone(), **co);
                                hidden_variable.push(h_map);
                            }
                        }
                    }
                }
            }
        }

        let h_name = format!("HIDDEN_{}", i);
        variables.insert_hidden(h_name.clone(), hidden_variable);

        // Unary Constraint for Hidden Variables (Addition Constraint)
        let (cvi, cvo) = (carry_in.clone(), carry_out.clone());
        constraints.push(Constraint::Unary((
            h_name.clone(),
            Rc::new(move |v| {
                if let VariableType::Hidden(h) = v {
                    let a = h.get(&c0.to_string()).copied().unwrap_or(0);
                    let b = h.get(&c1.to_string()).copied().unwrap_or(0);
                    let c = h.get(&c2.to_string()).copied().unwrap_or(0);
                    let ci = h.get(&cvi).copied().unwrap_or(0);
                    let co = h.get(&cvo).copied().unwrap_or(0);
                    return a + b + ci == c + 10 * co;
                }
                false
            }),
        )));

        // Binary Constraints: Hidden Variables <-> Original Variables
        for c in [c0.to_string(), c1.to_string(), c2.to_string(), carry_in.to_string(), carry_out.to_string()] {
            constraints.push(Constraint::Binary((
                c.clone(),
                h_name.clone(),
                Rc::new({
                    let c = c.to_string();
                    move |a, b| {
                        if let (VariableType::Value(x), VariableType::Hidden(h)) = (a, b) {
                            return *h.get(&c).unwrap() == *x;
                        }
                        false
                    }
                }),
            )));
        }
    }

    // Filter the domain
    filter_domain(&mut variables, &constraints);
    let assignment = solution(&variables, &constraints).unwrap();

    // Print Results
    println!("{:?}", assignment
        .iter()
        .filter_map(|(_, v)| v.value())
        .collect::<Vec<_>>());
}
    