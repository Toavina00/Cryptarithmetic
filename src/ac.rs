#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum VariableType<T> {
    Value(Rc<T>),
    Hidden(Rc<HashMap<String, T>>),
}

impl<T: Clone> VariableType<T> {
    pub fn value(&self) -> Option<&T> {
        if let VariableType::Value(x) = self {
            Some(&(**x))
        } else {
            None
        }
    }

    pub fn hidden(&self) -> Option<&HashMap<String, T>> {
        if let VariableType::Hidden(x) = self {
            Some(&(**x))
        } else {
            None
        }
    }

    pub fn value_ref(&self) -> Option<Rc<T>> {
        if let VariableType::Value(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }

    pub fn hidden_ref(&self) -> Option<Rc<HashMap<String, T>>> {
        if let VariableType::Hidden(x) = self {
            Some(x.clone())
        } else {
            None
        }
    }
}

pub type Assignement<T> = HashMap<String, VariableType<T>>;

pub struct Variables<T> {
    variable: HashMap<Rc<String>, Vec<VariableType<T>>>,
    var_names: HashSet<Rc<String>>,
    hid_names: HashSet<Rc<String>>,
}

impl<T: Clone> Variables<T> {
    pub fn new() -> Self {
        Variables {
            variable: HashMap::new(),
            var_names: HashSet::new(),
            hid_names: HashSet::new(),
        }
    }

    pub fn insert(&mut self, key: &str, values: Vec<T>) {
        let key = Rc::new(key.to_owned());
        self.variable.insert(
            key.clone(),
            values
                .into_iter()
                .map(Rc::from)
                .map(VariableType::Value)
                .collect(),
        );
        self.var_names.insert(key.clone());
    }

    pub fn insert_hidden(&mut self, key: &str, values: Vec<HashMap<String, T>>) {
        let key = Rc::new(key.to_owned());
        self.variable.insert(
            key.clone(),
            values
                .into_iter()
                .map(Rc::from)
                .map(VariableType::Hidden)
                .collect(),
        );
        self.hid_names.insert(key.clone());
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Rc<String>, &Vec<VariableType<T>>)> {
        self.variable
            .iter()
            .filter(|(k, _)| self.var_names.contains(*k))
    }

    pub fn iter_hidden(&self) -> impl Iterator<Item = (&Rc<String>, &Vec<VariableType<T>>)> {
        self.variable
            .iter()
            .filter(|(k, _)| self.hid_names.contains(*k))
    }

    pub fn values(&self) -> Vec<(&Rc<String>, Vec<&T>)> {
        self.iter()
            .map(|(k, v)| (k, v.iter().map(|x| x.value().unwrap()).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
    }

    pub fn hidden_values(&self) -> Vec<(&Rc<String>, Vec<&HashMap<String, T>>)> {
        self.iter_hidden()
            .map(|(k, v)| (k, v.iter().map(|x| x.hidden().unwrap()).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
    }

    pub fn get(&self, key: &str) -> Option<&Vec<VariableType<T>>> {
        self.variable.get(&Rc::new(key.to_owned()))
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Vec<VariableType<T>>> {
        self.variable.get_mut(&Rc::new(key.to_owned()))
    }

    pub fn names(&self) -> Vec<Rc<String>> {
        self.variable.keys().cloned().collect()
    }
}

pub enum Constraint<T> {
    Binary(
        (
            String,
            String,
            Rc<dyn Fn(VariableType<T>, VariableType<T>) -> bool>,
        ),
    ),
    Unary((String, Rc<dyn Fn(VariableType<T>) -> bool>)),
}

pub type Constraints<T> = Vec<Constraint<T>>;

fn arc_consistency<T: Clone>(variables: &mut Variables<T>, arcs: &Constraints<T>) {
    for arc in arcs {
        match arc {
            Constraint::Binary((x0, x1, a)) => {
                let x1_values = variables.get(x1).unwrap().clone();
                if let Some(x0_values) = variables.get_mut(x0) {
                    x0_values.retain(|x| x1_values.iter().any(|y| a(x.clone(), y.clone())));
                }
            }
            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get_mut(x) {
                    x_values.retain(|x| a(x.clone()));
                }
            }
        }
    }
}

fn is_consistent<T: Clone>(variables: &Variables<T>, constraints: &Constraints<T>) -> bool {
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
                } else {
                    return false;
                }
            }
            Constraint::Unary((x, a)) => {
                if let Some(x_values) = variables.get(x) {
                    if x_values.iter().any(|v| !a(v.clone())) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
    true
}

fn build_arcs<T: 'static>(constraints: &Constraints<T>) -> Constraints<T> {
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

pub fn filter_domain<T: Clone + 'static>(
    variables: &mut Variables<T>,
    constraints: &Constraints<T>,
) {
    let arcs = build_arcs(&constraints);

    loop {
        arc_consistency(variables, &arcs);
        if is_consistent(variables, constraints) {
            break;
        }
    }
}

fn is_solution<T: Clone>(assignement: &Assignement<T>, constraints: &Constraints<T>) -> bool {
    for constraint in constraints.iter() {
        match constraint {
            Constraint::Binary((x0, x1, a)) => {
                if let (Some(x), Some(y)) = (assignement.get(x0), assignement.get(x1)) {
                    if !a(x.clone(), y.clone()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            Constraint::Unary((x, a)) => {
                if let Some(v) = assignement.get(x) {
                    if !a(v.clone()) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
    true
}

fn backtrack<T: Clone>(
    assignement: &mut Assignement<T>,
    variables: &Variables<T>,
    constraints: &Constraints<T>,
    keys: &Vec<Rc<String>>,
    index: usize,
) -> bool {
    if is_solution(assignement, constraints) {
        return true;
    }

    if let Some(key) = keys.get(index) {
        if let Some(values) = variables.get(key) {
            for v in values {
                assignement.insert(key.to_string(), v.clone());

                if backtrack(assignement, variables, constraints, keys, index + 1) {
                    return true;
                }
            }
        }
    }

    false
}

pub fn solution<T: Clone>(
    variables: &Variables<T>,
    constraints: &Constraints<T>,
) -> Option<Assignement<T>> {
    let mut assignement = Assignement::new();

    if backtrack(
        &mut assignement,
        variables,
        constraints,
        &variables.names(),
        0,
    ) {
        Some(assignement)
    } else {
        None
    }
}
