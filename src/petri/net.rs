use crate::petri::data::DataTag;
use crate::petri::data::{Data, Query};
use crate::petri::matrix::MatrixNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::Transition;
use colorous::Color;
use colorous::SET3;
use colorous::TABLEAU10;
use enum_tag::EnumTag;
use inline_xml::{xml, xml_tag, Content, Tag, Xml};
use itertools::Itertools;
use nalgebra::DMatrix;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64::MIN;
use uuid::Uuid;

use super::transition::Signature;

pub fn random_agent_color(index: u8) -> Color {
    TABLEAU10[index as usize % TABLEAU10.len()]
}

pub fn random_task_color(index: u8) -> Color {
    SET3[index as usize % SET3.len()]
}

pub fn random_color(index: u8) -> Color {
    if index < 12 {
        SET3[index as usize % SET3.len()]
    } else if index < 22 {
        TABLEAU10[index as usize % TABLEAU10.len()]
    } else {
        TABLEAU10[index as usize % TABLEAU10.len()]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetriNet {
    pub id: Uuid,
    pub name: String,
    pub places: HashMap<Uuid, Place>,
    pub transitions: HashMap<Uuid, Transition>,
    pub initial_marking: HashMap<Uuid, usize>,
    pub name_lookup: HashMap<Uuid, String>
}

impl PetriNet {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            places: HashMap::new(),
            transitions: HashMap::new(),
            initial_marking: HashMap::new(),
            name_lookup: HashMap::new()
        }
    }

    pub fn get_dot(&self) -> String {
        let colors: HashMap<String, Color> = HashMap::from_iter(
            self.name_lookup
                .values()
                .unique()
                .enumerate()
                .map(|(i, k)| (k.clone(), random_color(i as u8))),
        );

        let mut dot = String::from(&format!(
            "digraph {} {{\nbgcolor=\"transparent\"\nfontname=\"helvetica\"\n",
            self.name.replace(" ", "_")
        ));

        for place in self.places.values() {
            let mark: String;
            match place.tokens {
                TokenSet::Infinite => mark = "∞".to_string(),
                TokenSet::Finite => {
                    mark = self
                        .initial_marking
                        .get(&place.id)
                        .unwrap_or(&0)
                        .to_string()
                }
                TokenSet::Sink => mark = "sink".to_string(),
            }
            let background = get_color_from_data(place, &self.name_lookup, &colors);
            dot.push_str(&format!("// Place {}\n", place.name));
            dot.push_str(&format!(
                "\t{} [label=<{}>,fillcolor=\"#{:X}\",style=filled,shape=circle,fontname=\"helvetica\"];\n",
                place.id.as_u128(),
                self.get_label_table(place.name.clone(),Some(mark),place.meta_data.clone(),&colors),
                background
            ));
        }

        for transition in self.transitions.values() {
            dot.push_str(&format!("// Transition {}\n", transition.name));
            dot.push_str(&format!(
                "\t{} [label=<{}>,shape=box,style=filled,fillcolor=\"#000000\",fontname=\"helvetica\"];\n",
                transition.id.as_u128(),
                self.get_label_table(transition.name.clone(),None,transition.meta_data.clone(),&colors)
            ));
        }

        for (id, transition) in self.transitions.iter() {
            for (place_id, signature) in transition.input.iter() {
                let place = self.places.get(place_id).unwrap();
                let line_color: Color = get_color_from_data(place, &self.name_lookup, &colors);

                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{:?}\",color=\"#{:X}\",fontcolor=\"#{:X}\",fontname=\"helvetica\",penwidth=10];\n",
                    place_id.as_u128(),
                    id.as_u128(),
                    signature,
                    line_color,
                    line_color
                ));
            }
            for (place_id, signature) in transition.output.iter() {
                let place = self.places.get(place_id).unwrap();
                let line_color: Color = get_color_from_data(place, &self.name_lookup, &colors);
                dot.push_str(&format!(
                    "\t{} -> {} [label=\"{:?}\",color=\"#{:X}\",fontcolor=\"#{:X}\",fontname=\"helvetica\",penwidth=10];\n",
                    id.as_u128(),
                    place_id.as_u128(),
                    signature,
                    line_color,
                    line_color
                ));
            }
        }

        dot.push_str("overlap=false\n");
        dot.push_str("}");
        dot
    }

    pub fn get_label_table(
        &self,
        title: String,
        marking: Option<String>,
        data: Vec<Data>,
        colors: &HashMap<String, Color>,
    ) -> String {
        let mut x = xml! {
               <TABLE BORDER="0" CELLPADDING="7">
                 <TR>
                      <TD COLSPAN="3" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                           <FONT POINT-SIZE="64" COLOR="#333333"><B>{title.clone()}</B></FONT>
                      </TD>
                 </TR>
               </TABLE>
        };
        match marking {
            Some(marking) => {
                let t = xml_tag! {
                    <TR>
                        <TD COLSPAN="3" ALIGN="text" BGCOLOR="#222222" STYLE="rounded">
                            <FONT POINT-SIZE="32" COLOR="#dddddd">{marking}</FONT>
                        </TD>
                    </TR>
                };
                add_tag_to_xml(&mut x, t);
            }
            None => {}
        }
        // return x.to_string();

        let default_name = "Unknown".to_string();
        let default_color: Color = Color {
            r: 240,
            g: 240,
            b: 240,
        };
        for data in data.iter() {
            let tag = data.tag();
            match (data.id(), data.secondary(), data.numeric()) {
                (None, None, None) => {
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="8" ALIGN="text" BGCOLOR="#555555" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#ffffff">{format!("{:?}",&tag)}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                }
                (Some(id), None, None) => {
                    let name = self.name_lookup.get(&id).unwrap_or(&default_name);
                    let bgcolor = colors.get(name).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="6" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                }
                (None, Some(id), None) => {
                    // This should never happen, but just in case, follow the same protocol as above
                    let name = self.name_lookup.get(&id).unwrap_or(&default_name);
                    let bgcolor = colors.get(name).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="6" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                }
                (Some(id1), Some(id2), None) => {
                    let name1 = self.name_lookup.get(&id1).unwrap_or(&default_name);
                    let name2 = self.name_lookup.get(&id2).unwrap_or(&default_name);
                    let bgcolor1 = colors.get(name1).unwrap_or(&default_color);
                    let bgcolor2 = colors.get(name2).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor1)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name1}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor2)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name2}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                },
                (Some(id1), Some(id2), Some(numeric)) => {
                    let name1 = self.name_lookup.get(&id1).unwrap_or(&default_name);
                    let name2 = self.name_lookup.get(&id2).unwrap_or(&default_name);
                    let bgcolor1 = colors.get(name1).unwrap_or(&default_color);
                    let bgcolor2 = colors.get(name2).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor1)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name1}</FONT>
                            </TD>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor2)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name2}</FONT>
                            </TD>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{}",numeric)}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                },
                (None, None, Some(numeric)) => {
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="6" ALIGN="text" BGCOLOR="#f0f0f0" BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{}",numeric)}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                },
                (Some(id), None, Some(numeric)) => {
                    let name = self.name_lookup.get(&id).unwrap_or(&default_name);
                    let bgcolor = colors.get(name).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR="#f0f0f0" BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{}",numeric)}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                },
                (None, Some(id), Some(numeric)) => {
                    // This should never happen, but just in case, follow the same protocol as above
                    let name = self.name_lookup.get(&id).unwrap_or(&default_name);
                    let bgcolor = colors.get(name).unwrap_or(&default_color);
                    let dt = xml_tag! {
                        <TR>
                            <TD COLSPAN="2" ALIGN="text" BGCOLOR="#f0f0f0" STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{:?}",&tag)}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR={format!("#{:X}",bgcolor)} BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{name}</FONT>
                            </TD>
                            <TD COLSPAN="3" ALIGN="text" BGCOLOR="#f0f0f0" BORDER="4"  STYLE="rounded">
                                <FONT POINT-SIZE="32" COLOR="#333333">{format!("{}",numeric)}</FONT>
                            </TD>
                        </TR>
                    };
                    add_tag_to_xml(&mut x, dt);
                }
            }
        }
        return x.to_string();
    }

    pub fn data_to_label(&self, data: &Data) -> String {
        match data.id() {
            None => return format!("{:?}", data.tag()),
            Some(id) => {
                return format!(
                    "{:?}:{}",
                    data.tag(),
                    self.name_lookup.get(&id).unwrap_or(&"Unknown".to_string())
                )
            }
        }
    }

    pub fn query_transitions(&self, query_vec: &Vec<Query>) -> Vec<&Transition> {
        self.transitions
            .values()
            .filter(|transition| transition.has_data(query_vec))
            .collect()
    }

    pub fn query_places(&self, query_vec: &Vec<Query>) -> Vec<&Place> {
        self.places
            .values()
            .filter(|place| place.has_data(query_vec))
            .collect()
    }

    pub fn delete_transition(&mut self, id: Uuid) {
        if self.transitions.contains_key(&id) {
            self.transitions.remove(&id);
        } else {
            println!("Transition with id {} does not exist", id);
        }
    }

    pub fn delete_place(&mut self, id: Uuid) {
        if self.places.contains_key(&id) {
            self.places.remove(&id);
            self.transitions.iter_mut().for_each(|(_, transition)| {
                transition.input.remove(&id);
                transition.output.remove(&id);
            });
        } else {
            println!("Place with id {} does not exist", id);
        }
    }

    pub fn add_transition_with_edge_conditions<FnIn, FnOut>(
        &mut self,
        name: String,
        meta_data: Vec<Data>,
        input_validator: FnIn,
        output_validator: FnOut,
        input_sig: Signature,
        output_sig: Signature,
    ) where
        FnIn: Fn(&Place) -> bool,
        FnOut: Fn(&Place) -> bool,
    {
        let input: HashMap<Uuid, Signature> = self
            .places
            .iter()
            .filter(|(_, place)| input_validator(place))
            .map(|(id, _)| (*id, input_sig.clone()))
            .collect();
        let output: HashMap<Uuid, Signature> = self
            .places
            .iter()
            .filter(|(_, place)| output_validator(place))
            .map(|(id, _)| (*id, output_sig.clone()))
            .collect();
        let transition = Transition::new(name, input, output, meta_data, 0.0, vec![]);
        self.transitions.insert(transition.id, transition);
    }

    pub fn split_place<EvalFn>(
        &mut self,
        id: &Uuid,
        splits: Vec<Vec<Data>>,
        eval_fn: EvalFn,
    ) -> (Vec<Uuid>, Vec<Uuid>)
    where
        EvalFn: Fn(&Transition, &Vec<Data>) -> bool,
    {
        let mut new_places: Vec<Uuid> = Vec::new();
        let mut new_transitions: Vec<Uuid> = Vec::new();
        if self.places.contains_key(id) {
            let template_place = self.places.get(id).unwrap().clone();
            let transition_neighbors: Vec<Uuid> = self
                .transitions
                .iter()
                .filter(|(_, transition)| {
                    transition.input.contains_key(id) || transition.output.contains_key(id)
                })
                .map(|(id, _)| *id)
                .collect();
            let mut removed_transitions: Vec<Uuid> = Vec::new();
            for split in splits {
                // println!("Splitting place {:?} for {:?}", id, split);
                let mut new_place = template_place.clone();
                let new_place_id = Uuid::new_v4();
                new_place.id = new_place_id;
                for split_data in split.iter() {
                    new_place.meta_data.push(split_data.clone());
                }
                self.places.insert(new_place_id, new_place);
                new_places.push(new_place_id);

                for transition_id in transition_neighbors.iter() {
                    let existing_transition = self.transitions.get(transition_id).unwrap();
                    // println!("Checking transition {:?}", existing_transition);
                    // println!("Eval fn: {:?}", eval_fn(existing_transition,&split));
                    if eval_fn(existing_transition, &split) {
                        let mut new_transition = existing_transition.clone();
                        new_transition.id = Uuid::new_v4();
                        let new_transition_id = new_transition.id.clone();
                        if new_transition.input.contains_key(id) {
                            new_transition.input.remove(id);
                            new_transition.input.insert(new_place_id, Signature::Static(1));
                        }
                        if new_transition.output.contains_key(id) {
                            new_transition.output.remove(id);
                            new_transition.output.insert(new_place_id, Signature::Static(1));
                        }
                        for split_data in split.iter() {
                            if !new_transition.meta_data.contains(split_data) {
                                new_transition.meta_data.push(split_data.clone());
                            }
                        }
                        self.transitions.insert(new_transition.id, new_transition);
                        new_transitions.push(new_transition_id);
                        if !removed_transitions.contains(transition_id) {
                            removed_transitions.push(*transition_id);
                        }
                    }
                    if !removed_transitions.contains(transition_id) {
                        removed_transitions.push(*transition_id);
                    }
                }
            }
            for transition_id in removed_transitions.iter() {
                self.transitions.remove(transition_id);
            }
            self.places.remove(id);
        }

        (new_places, new_transitions)
    }

    pub fn transitions_derived_from_task(&mut self, task: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::Task(task)))
            .collect()
    }

    pub fn transitions_associated_with_agent(&mut self, agent: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| transition.meta_data.contains(&Data::Agent(agent)))
            .collect()
    }

    pub fn transitions_connected_to_place_mut(&mut self, place: Uuid) -> Vec<&mut Transition> {
        self.transitions
            .values_mut()
            .into_iter()
            .filter(|transition| {
                transition.input.contains_key(&place) || transition.output.contains_key(&place)
            })
            .collect()
    }

    pub fn transitions_connected_to_place(&mut self, place: Uuid) -> Vec<&Transition> {
        self.transitions
            .values()
            .into_iter()
            .filter(|transition| {
                transition.input.contains_key(&place) || transition.output.contains_key(&place)
            })
            .collect()
    }
}

fn add_tag_to_xml(x: &mut Xml, t: Tag) {
    let content = x.0.iter_mut().last().unwrap();
    match content {
        Content::Tag(ref mut inner_t) => match inner_t.inner {
            Some(ref mut inner) => {
                inner.0.push(Content::Tag(t));
            }
            None => {
                inner_t.inner = Some(Xml(vec![Content::Tag(t)]));
            }
        },
        _ => {}
    }
}

fn get_color_from_data(place: &Place, name_lookup: &HashMap<Uuid, String>, colors: &HashMap<String,Color>) -> Color {
    let mut color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    let color_data: Vec<&Data> = place
        .meta_data
        .iter()
        .filter(|d| d.tag() == DataTag::Agent || d.tag() == DataTag::Target)
        .collect();
    if color_data.len() == 1 {
        let name = name_lookup.get(&color_data[0].id().unwrap()).unwrap();
        color = colors.get(name).unwrap().clone();
    } else if color_data.len() == 2 {
        let mut current_tag = DataTag::AgentAgnostic;
        let duplicated_data = vec![color_data.clone(),color_data.clone()].into_iter().concat();
        for data in duplicated_data.iter() {
            if data.tag() == DataTag::Agent
                && (current_tag == DataTag::AgentAgnostic
                    || current_tag == DataTag::Target
                    || current_tag == DataTag::AgentJoint)
            {
                let name = name_lookup.get(&data.id().unwrap()).unwrap();
                color = colors.get(name).unwrap().clone();
                current_tag = data.tag();
            } else if data.tag() == DataTag::Target
                && (current_tag == DataTag::AgentAgnostic
                    || current_tag == DataTag::AgentJoint)
            {
                let name = name_lookup.get(&data.id().unwrap()).unwrap();
                color = colors.get(name).unwrap().clone();
                current_tag = data.tag();
            } else if data.tag() == DataTag::Agent && current_tag == DataTag::Agent {
                color = Color {
                    r: 255,
                    g: 255,
                    b: 255,
                };
                current_tag = DataTag::AgentJoint;
            }
        }
    }
    color
}

#[test]
fn add_transition_with_edge_conditions() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Task(id2)]);
    let p2 = Place::new(
        "P2".into(),
        TokenSet::Finite,
        vec![Data::Task(id2), Data::Agent(id1)],
    );
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Task(id3)]);
    net.places.insert(p1.id, p1.clone());
    net.places.insert(p2.id, p2.clone());
    net.places.insert(p3.id, p3.clone());
    net.add_transition_with_edge_conditions(
        "Added Transition".into(),
        vec![Data::AgentAgnostic],
        |place: &Place| {
            !place.has_data(&vec![Query::Data(Data::Agent(id1))])
                && place.has_data(&vec![Query::Data(Data::Task(id2))])
        },
        |place: &Place| {
            !place.has_data(&vec![Query::Data(Data::Agent(id1))])
                && place.has_data(&vec![Query::Data(Data::Task(id3))])
        },
        Signature::Static(1),
        Signature::Static(2),
    );
    println!("{:#?}", net);
    assert_eq!(net.transitions.len(), 1);
    net.transitions.iter().for_each(|(_id, transition)| {
        assert_eq!(transition.name, "Added Transition");
        assert_eq!(transition.input.len(), 1);
        assert_eq!(transition.output.len(), 1);
        assert_eq!(
            transition.input.get(&p1.id).unwrap_or(&Signature::Static(0)),
            &Signature::Static(1)
        );
        assert_eq!(
            transition.output.get(&p3.id).unwrap_or(&Signature::Static(0)),
            &Signature::Static(2)
        );
        assert_eq!(transition.meta_data, vec![Data::AgentAgnostic]);
    });
}

#[test]
fn split_place() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();
    let id5 = Uuid::new_v4();
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Task(id1)]);
    let p2 = Place::new("P2".into(), TokenSet::Finite, vec![Data::Task(id2)]);
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Task(id3)]);
    let t1 = Transition::new(
        "T2".into(),
        HashMap::from([(p1.id, Signature::Static(1))]),
        HashMap::from([(p2.id, Signature::Static(1))]),
        vec![],
        0.0,
        vec![],
    );
    let t2 = Transition::new(
        "T2".into(),
        HashMap::from([(p2.id, Signature::Static(1))]),
        HashMap::from([(p3.id, Signature::Static(1))]),
        vec![],
        0.0,
        vec![],
    );
    let p2_id = p2.id;
    net.places.insert(p1.id, p1);
    net.places.insert(p2_id, p2);
    net.places.insert(p3.id, p3);
    net.transitions.insert(t1.id, t1);
    net.transitions.insert(t2.id, t2);
    net.split_place(
        &p2_id,
        vec![
            vec![Data::Agent(id4), Data::AgentSituated(id4)],
            vec![Data::Agent(id5)],
        ],
        |_transition, _split_data| true,
    );
    println!("{:#?}", net);
    assert_eq!(net.places.len(), 4);
    assert_eq!(net.transitions.len(), 4);
    assert_eq!(
        net.query_transitions(&vec![
            Query::Data(Data::Agent(id4)),
            Query::Data(Data::AgentSituated(id4))
        ])
        .first()
        .unwrap()
        .input
        .len(),
        1
    );
    assert_eq!(
        net.query_transitions(&vec![
            Query::Data(Data::Agent(id4)),
            Query::Data(Data::AgentSituated(id4))
        ])
        .first()
        .unwrap()
        .output
        .len(),
        1
    );
}

#[test]
fn filtered_split_place() {
    let mut net = PetriNet::new("Test".into());
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();
    let agent_id = Uuid::new_v4();
    let initial = Place::new("Initial".into(), TokenSet::Finite, vec![]);
    let center = Place::new("center".into(), TokenSet::Finite, vec![Data::Target(id1)]);
    let p1 = Place::new("P1".into(), TokenSet::Finite, vec![Data::Hand(id2,agent_id)]);
    let p2 = Place::new("P2".into(), TokenSet::Finite, vec![Data::Hand(id3,agent_id)]);
    let p3 = Place::new("P3".into(), TokenSet::Finite, vec![Data::Hand(id4,agent_id)]);
    

    let center_id = center.id;

    let t0 = Transition::new(
        "T0".into(),
        HashMap::from([(initial.id, Signature::Static(1))]),
        HashMap::from([(center_id, Signature::Static(1))]),
        vec![],
        0.0,
        vec![],
    );

    let t1 = Transition::new(
        "T1".into(),
        HashMap::from([(center_id, Signature::Static(1))]),
        HashMap::from([(p1.id, Signature::Static(1))]),
        vec![Data::Hand(id2,agent_id)],
        0.0,
        vec![],
    );

    let t2 = Transition::new(
        "T2".into(),
        HashMap::from([(center_id, Signature::Static(1))]),
        HashMap::from([(p2.id, Signature::Static(1))]),
        vec![Data::Hand(id3,agent_id)],
        0.0,
        vec![],
    );

    let t3 = Transition::new(
        "T3".into(),
        HashMap::from([(center_id, Signature::Static(1))]),
        HashMap::from([(p3.id, Signature::Static(1))]),
        vec![Data::Hand(id4,agent_id)],
        0.0,
        vec![],
    );

    let t4 = Transition::new(
        "T4".into(),
        HashMap::from([(p1.id, Signature::Static(1))]),
        HashMap::from([(center_id, Signature::Static(1))]),
        vec![Data::Hand(id2,agent_id)],
        0.0,
        vec![],
    );

    let t5 = Transition::new(
        "T5".into(),
        HashMap::from([(p2.id, Signature::Static(1))]),
        HashMap::from([(center_id, Signature::Static(1))]),
        vec![Data::Hand(id3,agent_id)],
        0.0,
        vec![],
    );

    let t6 = Transition::new(
        "T6".into(),
        HashMap::from([(p3.id, Signature::Static(1))]),
        HashMap::from([(center_id, Signature::Static(1))]),
        vec![Data::Hand(id4,agent_id)],
        0.0,
        vec![],
    );

    net.places.insert(initial.id, initial);
    net.places.insert(center.id, center);
    net.places.insert(p1.id, p1);
    net.places.insert(p2.id, p2);
    net.places.insert(p3.id, p3);
    net.transitions.insert(t0.id, t0);
    net.transitions.insert(t1.id, t1);
    net.transitions.insert(t2.id, t2);
    net.transitions.insert(t3.id, t3);
    net.transitions.insert(t4.id, t4);
    net.transitions.insert(t5.id, t5);
    net.transitions.insert(t6.id, t6);

    net.name_lookup.insert(id1, "Target".into());
    net.name_lookup.insert(id2, "Hand1".into());
    net.name_lookup.insert(id3, "Hand2".into());
    net.name_lookup.insert(id4, "Hand3".into());

    // println!("{:#?}", net);

    net.split_place(
        &center_id,
        vec![vec![Data::Standing(id2,agent_id)], vec![Data::Standing(id3,agent_id)]],
        |transition, split_data| {
            if transition.meta_data.len() == 0 {
                return true;
            }
            let poi = split_data
                .iter()
                .find(|data| data.tag() == DataTag::Standing)
                .unwrap();
            let hand = transition
                .meta_data
                .iter()
                .find(|data| data.tag() == DataTag::Hand)
                .unwrap();
            poi.id() == hand.id()
        },
    );

    // println!("{:#?}", net);

    assert_eq!(net.places.len(), 6);
    assert_eq!(net.transitions.len(), 6);
    assert_eq!(
        net.query_places(&vec![Query::Data(Data::Standing(id2,agent_id))])
            .len(),
        1
    );
    assert_eq!(
        net.query_places(&vec![Query::Data(Data::Hand(id2,agent_id))]).len(),
        1
    );
    assert_eq!(
        net.query_places(&vec![Query::Data(Data::Standing(id3,agent_id))])
            .len(),
        1
    );
    assert_eq!(
        net.query_places(&vec![Query::Data(Data::Hand(id3,agent_id))]).len(),
        1
    );
    assert_eq!(
        net.query_transitions(&vec![Query::Data(Data::Hand(id2,agent_id))])
            .len(),
        2
    );
    assert_eq!(
        net.query_transitions(&vec![Query::Data(Data::Hand(id3,agent_id))])
            .len(),
        2
    );
    assert_eq!(
        net.query_transitions(&vec![Query::Data(Data::Standing(id2,agent_id))])
            .len(),
        3
    );
    assert_eq!(
        net.query_transitions(&vec![Query::Data(Data::Standing(id3,agent_id))])
            .len(),
        3
    );
    assert_eq!(
        net.query_transitions(&vec![Query::Data(Data::Hand(id4,agent_id))])
            .len(),
        0
    );
}
