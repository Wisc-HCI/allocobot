use itertools::Itertools;
use uuid::Uuid;
use z3::ast;
use z3::ast::Ast;
use z3::{Config, Context, Model, Optimize};
#[cfg(test)]
use crate::description::rating::Rating;
#[cfg(test)]
use std::collections::HashMap;
use nalgebra::{Vector2, Vector3};

use crate::description::primitive::Primitive;

pub fn split_primitives(primitives: &Vec<&Primitive>, splits: usize) -> Vec<Vec<Uuid>> {

    assert!(primitives.len() >= splits);

    let ctx: Context = Context::new(&Config::default());
    let optimizer = Optimize::new(&ctx);

    let assignments: Vec<ast::Int> = primitives.iter().map(|primitive| ast::Int::new_const(
        &ctx,
        format!("{}_assignment", primitive.id()),
    )).collect();

    // Create a set of weights. We will try to maximize these
    let mut weights:Vec<ast::Int> = Vec::new(); 

    // Add the constraints for each assignment
    assignments.iter().for_each(|assignment| {
        optimizer.assert(&assignment.ge(&ast::Int::from_i64(&ctx, 0)));
        optimizer.assert(&assignment.le(&ast::Int::from_i64(&ctx, splits as i64)));
    });

    // 
    // Add the constraints for each pair of assignments
    primitives.iter().enumerate().tuple_combinations().for_each(
        |(
            (primitive_idx1,primitive_1), 
            (primitive_idx2,primitive_2)
        )| {
            // println!("Affiliation between {:?} and {:?} is {} (match: {})", primitive_1, primitive_2, primitive_1.affiliation(primitive_2),primitive_1.target()==primitive_2.target());
            let weight = ast::Int::new_const(&ctx, format!("{}_{}_weight", primitive_1.id(), primitive_2.id()));
            optimizer.assert(&assignments[primitive_idx1]._eq(&assignments[primitive_idx2]).implies(&weight._eq(&ast::Int::from_i64(&ctx, primitive_1.affiliation(primitive_2) as i64))));
            optimizer.assert(&assignments[primitive_idx1]._eq(&assignments[primitive_idx2]).not().implies(&weight._eq(&ast::Int::from_i64(&ctx, 0))));
            weights.push(weight);
    });

    // Add the constraint that each split must have at least one member
    
    for split_idx in 0..splits {
        let equalities: Vec<ast::Bool> = assignments.iter().map(|a| a._eq(&ast::Int::from_i64(&ctx, split_idx as i64))).collect();
        optimizer.assert(&ast::Bool::pb_ge(&ctx, equalities.iter().map(|e| (e,1 as i32)).collect::<Vec<(&z3::ast::Bool<'_>, i32)>>().as_slice(), 1));
    }

    let total: ast::Int = ast::Int::add(&ctx, weights.iter().collect::<Vec<&ast::Int>>().as_slice());
    optimizer.maximize(&total);

    let _satisfied = optimizer.check(&[]);
    let model: Option<Model> = optimizer.get_model();

    // println!("Model: {:?}", model);

    let mut split_vec: Vec<Vec<Uuid>> = vec![];
    for _ in 0..splits {
        split_vec.push(vec![]);
    }

    match model {
        Some(m) => {
            let assignment_indices = assignments.iter().map(|a| m.eval(a, true).unwrap().as_i64().unwrap() as usize).collect::<Vec<usize>>();
            for (idx, primitive) in primitives.iter().enumerate() {
                split_vec[assignment_indices[idx]].push(primitive.id());
            }
        },
        None => {
            for (idx, primitive) in primitives.iter().enumerate() {
                split_vec[idx % splits].push(primitive.id());
            }
        }
    }
    split_vec
}

pub fn fitz_law(a: f64, b: f64, d: f64, w: f64) -> f64 {
    a + b * index_of_difficulty(d, w)
}

pub fn index_of_difficulty(d: f64, w: f64) -> f64 {
    (2.0 * d / w).log2()
}

pub fn vector3_distance_f64(vector1: Vector3<f64>, vector2: Vector3<f64>) -> f64 {
    return ((vector1.x - vector2.x).powf(2.0) + (vector1.y - vector2.y).powf(2.0) + (vector1.z - vector2.z).powf(2.0)).sqrt();
}

pub fn vector2_distance_f64(vector1: Vector2<f64>, vector2: Vector2<f64>) -> f64 {
    return ((vector1.x - vector2.x).powf(2.0) + (vector1.y - vector2.y).powf(2.0)).sqrt();
}

#[test]
pub fn test_split() {
    let target1 = Uuid::new_v4();
    let _target2 = Uuid::new_v4();

    let inspect = Primitive::Inspect { id: Uuid::new_v4(), target: target1, skill: Rating::High };
    let force = Primitive::Force { id: Uuid::new_v4(), target: target1, magnitude: 3.0 };
    let hold = Primitive::Hold { id: Uuid::new_v4(), target: target1 };
    let position = Primitive::Position { id: Uuid::new_v4(), target: target1, degrees: 180.0 };

    let primitives = vec![
        &inspect,
        &force,
        &hold,
        &position
    ];

    let mut lookup = HashMap::new();
    lookup.insert(inspect.id(), &inspect);
    lookup.insert(force.id(), &force);
    lookup.insert(hold.id(), &hold);
    lookup.insert(position.id(), &position);

    let splits = split_primitives(&primitives, 2);
    println!("Splits: {:#?}", splits.iter().map(|split| split.iter().map(|id| *lookup.get(id).unwrap()).collect::<Vec<&Primitive>>()).collect::<Vec<Vec<&Primitive>>>());

    assert!(splits.len() == 2);
    assert!(splits[0].len() == 3 || splits[1].len() == 3);
    assert!((splits[0].len() == 1 && splits[0].contains(&force.id())) || (splits[1].len() == 1 && splits[1].contains(&force.id())))
}
