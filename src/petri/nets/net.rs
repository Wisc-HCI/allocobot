use std::collections::HashMap;
// use std::error::Error;
use crate::petri::data::Data;
// use crate::description::target;
use crate::description::task::Task;
// use crate::description::target::Target;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use uuid::Uuid;
// use std::fmt;
use nalgebra::Vector3;
use rand::prelude::*;

pub fn color_to_hex(color: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", color.0, color.1, color.2)
}

pub fn random_color() -> (u8, u8, u8) {
    let mut rng = rand::thread_rng();
    normalize_color((rng.gen(), rng.gen(), rng.gen()))
}

pub fn normalize_color(color: (u8, u8, u8)) -> (u8, u8, u8) {
    let color_vec: Vector3<f32> = Vector3::new(color.0 as f32, color.1 as f32, color.2 as f32);
    let normalized = color_vec.normalize();
    // Bump up the brightness by multiplying by 280 instead of 255
    (
        (normalized.x * 280.0) as u8,
        (normalized.y * 280.0) as u8,
        (normalized.z * 280.0) as u8,
    )
}

pub trait PetriNet<'a> {
    fn get_id(&self) -> &Uuid;
    fn get_name(&self) -> &String;
    fn get_places(&self) -> HashMap<Uuid, &Place>;
    fn get_transitions(&self) -> HashMap<Uuid, &Transition>;
    fn get_tasks(&self) -> HashMap<Uuid, &Task<'a>>;
    fn get_places_mut(&mut self) -> HashMap<Uuid, &mut Place>;
    fn get_transitions_mut(&mut self) -> HashMap<Uuid, &mut Transition>;
    fn get_tasks_mut(&mut self) -> HashMap<Uuid, &mut Task<'a>>;
    fn get_random_initial_marking(&self) -> HashMap<Uuid, usize>;
    fn get_dot(&self) -> String {
        let mut colors: HashMap<Uuid, (u8, u8, u8)> = HashMap::new();

        let mut dot = String::from(&format!("digraph {} {{\n", self.get_name()));
        for place in self.get_places().values() {
            let mut background: (u8, u8, u8) = (255, 255, 255);
            place.meta_data.iter().for_each(|meta| match meta {
                Data::AgentLock(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    background = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",style=filled,fillcolor=\"{}\",penwidth=3];\n",
                place.id.as_u128(),
                place.name,
                color_to_hex(background)
            ));
        }

        for transition in self.get_transitions().values() {
            let mut font_color: (u8, u8, u8) = (255, 255, 255);
            let mut border_color: (u8, u8, u8) = (255, 255, 255);
            transition.meta_data.iter().for_each(|meta| match meta {
                Data::AgentTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    font_color = colors.get(id).unwrap().clone();
                }
                Data::TaskTransition(id) => {
                    if !colors.contains_key(id) {
                        colors.insert(*id, random_color());
                    }
                    border_color = colors.get(id).unwrap().clone();
                }
                _ => {}
            });
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!(
                "\t{} [label=\"{}\",shape=box,style=filled,fillcolor=\"#000000\",fontcolor=\"{}\",color=\"{}\",penwidth=3];\n",
                transition.id.as_u128(),
                transition.name,
                color_to_hex(font_color),
                color_to_hex(border_color)
            ));
        }
        
        for (id, transition) in self.get_transitions().iter() {
            for (place_id, count) in transition.input.iter() {
                let mut line_color: (u8, u8, u8) = (0, 0, 0);
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_color());
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"{}\",penwidth=3];\n",
                    place_id.as_u128(),
                    id.as_u128(),
                    count,
                    color_to_hex(line_color)
                ));
            }
            for (place_id, count) in transition.output.iter() {
                let mut line_color: (u8, u8, u8) = (0, 0, 0);
                transition.meta_data.iter().for_each(|meta| match meta {
                    Data::AgentTransition(id) => {
                        if !colors.contains_key(id) {
                            colors.insert(*id, random_color());
                        }
                        line_color = colors.get(id).unwrap().clone();
                    }
                    _ => {}
                });
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{}\",color=\"{}\",penwidth=3];\n",
                    id.as_u128(),
                    place_id.as_u128(),
                    count,
                    color_to_hex(line_color)
                ));
            }
        }

        dot.push_str("overlap=false\n");
        dot.push_str("}");
        dot
    }

    fn query_transitions(&self, meta_data: &Vec<Data>, fuzzy: bool) -> Vec<&Transition> {
        self.get_transitions()
            .values()
            .filter(|transition| transition.has_data(meta_data, fuzzy))
            .map(|transition| *transition)
            .collect()
    }

    fn query_places(&self, meta_data: &Vec<Data>, fuzzy: bool) -> Vec<&Place> {
        self.get_places()
            .values()
            .filter(|place| place.has_data(meta_data, fuzzy))
            .map(|place| *place)
            .collect()
    }
}
