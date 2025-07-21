#![allow(dead_code)]

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone)]
pub enum VariableType<T> {
    Value(Rc<T>),
    Hidden(Rc<HashMap<String, T>>),
}

pub type Assignement<T> = HashMap<String, T>;

pub struct Variables<T> {
    pub variable: HashMap<String, Vec<Rc<T>>>,
    pub hidden_variable: HashMap<String, Vec<Rc<HashMap<String, T>>>>,
}

impl<T: Clone> Variables<T> {
    pub fn new() -> Self {
        Variables {
            variable: HashMap::new(),
            hidden_variable: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, values: Vec<T>) {
        self.variable.insert(key, values.iter().cloned().map(Rc::from).collect());
    }

    pub fn insert_hidden(&mut self, key: String, values: Vec<HashMap<String, T>>) {
        self.hidden_variable.insert(key, values.iter().cloned().map(Rc::from).collect());
    }

    pub fn get(&self, key: &str) -> Option<Vec<VariableType<T>>> {
        if let Some(var) = self.variable.get(key) {
            Some(var.iter().cloned().map(VariableType::Value).collect())
        } else if let Some(hidden) = self.hidden_variable.get(key) {
            Some(hidden.iter().cloned().map(VariableType::Hidden).collect())
        } else {
            None
        }
    }

    pub fn update(&mut self, key: &str, values: impl Iterator<Item = VariableType<T>>) {
        if let Some(var) = self.variable.get_mut(key) {
            *var = values
                .filter_map(|x| match x {
                    VariableType::Value(v) => Some(v.clone()),
                    _ => None,
                })
                .collect();
        } else if let Some(var) = self.hidden_variable.get_mut(key) {
            *var = values
                .filter_map(|x| match x {
                    VariableType::Hidden(v) => Some(v.clone()),
                    _ => None,
                })
                .collect();
        }
    }
}

pub enum Constraint<T> {
    Binary((String, String, Rc<dyn Fn(VariableType<T>, VariableType<T>) -> bool>)),
    Unary((String, Rc<dyn Fn(VariableType<T>) -> bool>)),
}

pub type Constraints<T> = Vec<Constraint<T>>;

pub fn arc_consistency<T: Clone>(variables: &mut Variables<T>, arcs: &Constraints<T>) {
    for arc in arcs {
        match arc {
            Constraint::Binary((x0, x1, a)) => {
                if let (Some(x0_values), Some(x1_values)) = (variables.get(x0), variables.get(x1)) {
                    variables.update(
                        x0,
                        x0_values
                            .iter()
                            .filter(|&x| x1_values.iter().any(|y| a(x.clone(), y.clone())))
                            .cloned(),
                    );
                }
            }
            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get(x) {
                    variables.update(
                        x,
                        x_values.iter().filter(|&x| a(x.clone())).cloned(),
                    );
                }
            }
        }
    }
}

pub fn is_consistent<T: Clone>(variables: &Variables<T>, constraints: &Constraints<T>) -> bool {
    for constraint in constraints.iter() {
        match constraint {
            Constraint::Binary((x0, x1, a)) => {
                if let (Some(x0_values), Some(x1_values)) = (variables.get(x0), variables.get(x1)) {
                    if x0_values
                        .iter()
                        .any(|x| x1_values.iter().all(|y| !a(x.clone(), y.clone())))
                    {
                        return false;
                    }
                }
            }
            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get(x) {
                    if x_values.iter().any(|v| !a(v.clone())) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

pub fn build_arcs<T: 'static>(constraints: &Constraints<T>) -> Constraints<T> {
    let mut arcs = Vec::new();

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

pub fn filter_domain<T: Clone + 'static>(variables: &mut Variables<T>, constraints: &Constraints<T>) {
    let arcs = build_arcs(&constraints);

    loop {
        arc_consistency(variables, &arcs);
        if is_consistent(variables, constraints) {
            break;
        }
    }
}