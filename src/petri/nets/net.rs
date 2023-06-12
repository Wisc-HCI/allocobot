use std::collections::HashMap;
// use std::error::Error;

// use crate::description::target;
use crate::description::task::Task;
// use crate::description::target::Target;
use crate::petri::place::Place;
use crate::petri::transition::Transition;
use uuid::Uuid;
// use std::fmt;

pub trait PetriNet<'a> {
    fn get_id(&self) -> &Uuid;
    fn get_name(&self) -> &String;
    fn get_places(&self) -> HashMap<Uuid, &Place>;
    fn get_transitions(&self) -> HashMap<Uuid, &Transition>;
    fn get_tasks(&self) -> HashMap<Uuid, &Task<'a>>;
    fn get_places_mut(&mut self) -> HashMap<Uuid, &mut Place>;
    fn get_transitions_mut(&mut self) -> HashMap<Uuid, &mut Transition>;
    fn get_tasks_mut(&mut self) -> HashMap<Uuid, &mut Task<'a>>;
    fn get_dot(&self) -> String {
        
        let mut dot = String::from(&format!("digraph {} {{\n", self.get_name()));
        for place in self.get_places().values() {
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!("\t{} [label=\"{}\"];\n", place.id.as_u128(), place.name));
        }
        for transition in self.get_transitions().values() {
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!("\t{} [label=\"{}\",shape=box];\n", transition.id.as_u128(), transition.name));
        }
        for (id, transition) in self.get_transitions().iter() {
            for (place_id, count) in transition.input.iter() {
                dot.push_str(&format!("\t{} -> {} [label=\"{}\"];\n", place_id.as_u128(), id.as_u128(), count));
            }
            for (place_id, count) in transition.output.iter() {
                dot.push_str(&format!("\t{} -> {} [label=\"{}\"];\n", id.as_u128(), place_id.as_u128(), count));
            }
        }
        dot.push_str("}");
        dot
    }
}

// impl <'a> fmt::Display for dyn PetriNet<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "BasicNet {} ({}): {{\n", self.get_name(), self.get_id())?;
//         if self.get_places().is_empty() {
//             write!(f, "\tPlaces: [],\n")?;
//         } else {
//             write!(f, "\tPlaces: [\n")?;
//             for place in self.get_places().values() {
//                 write!(f, "\t\t{}: {{ name: {}, tokens: {:?}, source_task: {:?}}},\n", place.id, place.name, place.tokens, place.source_task)?;
//             }
//             write!(f, "\t],\n")?;
//         }

//         if self.get_transitions().is_empty() {
//             write!(f, "\tTransitions: [],\n")?;
//         } else {
//             write!(f, "\tTransitions: [\n")?;
//             for transition in self.get_transitions().values() {
//                 write!(f, "\t\t{}: {{ name: {}, input: {:?}, output: {:?}, source_task: {:?}}},\n", transition.id, transition.name, transition.input,  transition.output, transition.source_task)?;
//             }
//             write!(f, "\t],\n")?;
//         }
       
//         write!(f, "}}\n")
//     }
// }
