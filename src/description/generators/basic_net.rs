use crate::description::job::Job;
use crate::description::target::Target;
use crate::petri::data::{Data, Query};
use crate::petri::net::PetriNet;
use crate::petri::place::Place;
use crate::petri::token::TokenSet;
use crate::petri::transition::{Signature, Transition};
use enum_tag::EnumTag;
use std::collections::HashMap;
use uuid::Uuid;

impl Job {
    pub fn create_basic_net(&mut self) -> PetriNet {
        let mut net: PetriNet = PetriNet::new(self.name.clone());

        for (target_id, target) in self.targets.iter() {
            match target {
                Target::Product { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id), Data::TargetLocationSelected(*target_id), Data::TargetSituated(*target_id)],
                    );

                    let sink = Place::new(
                        format!("Target: {} (sink)", name),
                        TokenSet::Sink,
                        vec![Data::Target(*target_id), Data::TargetUnplaced(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    let sink_id = sink.id;
                    net.places.insert(sink_id, sink);
                    net.initial_marking.insert(place_id, 0);
                    net.initial_marking.insert(sink_id, 0);

                    let sink_transition = Transition::new(
                        format!("Sink Part: {}", name),
                        vec![(place_id, Signature::Static(1))]
                            .into_iter()
                            .collect(),
                        vec![(sink_id, Signature::Static(1))].into_iter().collect(),
                        vec![
                            Data::Simulation,
                            Data::Target(*target_id),
                            Data::TargetUnplaced(*target_id),
                            Data::TargetLocationSelected(*target_id),
                            Data::AgentAgnostic,
                            Data::Produce(*target_id, target.value()),
                        ],
                        0.0,
                        vec![],
                    );
                    net.transitions
                        .insert(sink_transition.id, sink_transition);
                }
                Target::Intermediate { name, .. } => {
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id), Data::TargetSituated(*target_id)],
                    );
                    let place_id = place.id;
                    net.places.insert(place_id, place);
                    net.initial_marking.insert(place_id, 0);
                }
                Target::Precursor { name, .. } => {
                    let infinite_source = Place::new(
                        format!("Target: {} (source)", name),
                        TokenSet::Infinite,
                        vec![Data::Target(*target_id), Data::TargetUnplaced(*target_id)],
                    );

                    let spawn = Place::new(
                        format!("Target: {} (spawn)", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id), Data::TargetLocationSelected(*target_id), Data::TargetSituated(*target_id)],
                    );

                    let infinite_source_id = infinite_source.id;
                    let spawn_id = spawn.id;
                    net.places.insert(spawn_id, spawn);
                    net.places.insert(infinite_source_id, infinite_source);
                    net.initial_marking.insert(infinite_source_id, 0);
                    let spawn_transition = Transition::new(
                        format!("Spawn Part: {}", name),
                        vec![(infinite_source_id, Signature::Static(1))]
                            .into_iter()
                            .collect(),
                        vec![(spawn_id, Signature::Static(1))].into_iter().collect(),
                        vec![
                            Data::Simulation,
                            Data::Target(*target_id),
                            Data::TargetSituated(*target_id),
                            Data::TargetLocationSelected(*target_id),
                            Data::AgentAgnostic,
                            Data::Spawn(*target_id, target.value()),
                        ],
                        0.0,
                        vec![],
                    );
                    net.transitions
                        .insert(spawn_transition.id, spawn_transition);
                }
                Target::Reusable { name, .. } => {
                    let pre_place = Place::new(
                        format!("Target: {} (pre)", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id), Data::TargetUnplaced(*target_id)],
                    );
                    let place = Place::new(
                        format!("Target: {}", name),
                        TokenSet::Finite,
                        vec![Data::Target(*target_id), Data::TargetSituated(*target_id)],
                    );
                    let place_id = place.id;
                    let pre_place_id = pre_place.id;
                    net.places.insert(place_id, place);
                    net.places.insert(pre_place_id, pre_place);
                    net.initial_marking.insert(pre_place_id, 1);
                    let situate_transition = Transition::new(
                        format!("Situate: {}", name),
                        vec![(pre_place_id, Signature::Static(1))]
                            .into_iter()
                            .collect(),
                        vec![(place_id, Signature::Static(1))].into_iter().collect(),
                        vec![
                            Data::Setup,
                            Data::Target(*target_id),
                            Data::TargetSituated(*target_id),
                            Data::AgentAgnostic,
                        ],
                        0.0,
                        vec![],
                    );
                    net.transitions
                        .insert(situate_transition.id, situate_transition);
                }
            }
            net.name_lookup.insert(*target_id, target.name());
        }

        // Add all dependencies as transitions to the net
        for (task_id, task) in self.tasks.iter() {
            net.name_lookup.insert(*task_id, task.name.clone());
            let mut input: HashMap<Uuid, Signature> = HashMap::new();
            let mut output: HashMap<Uuid, Signature> = HashMap::new();
            for (dependency_id, count) in &task.dependencies {
                let target_places = net.query_places(&vec![
                    Query::Data(Data::Target(*dependency_id)),
                    Query::Data(Data::TargetSituated(*dependency_id)),
                ]);
                for target_place in target_places {
                    input.insert(target_place.id, Signature::Static(*count));
                }
            }
            for (output_id, count) in &task.output {
                let target_places = net.query_places(&vec![
                    Query::Data(Data::Target(*output_id)),
                    Query::Data(Data::TargetSituated(*output_id)),
                ]);
                for target_place in target_places {
                    output.insert(target_place.id, Signature::Static(*count));
                }
            }

            let transition: Transition = Transition::new(
                format!("{}", task.name),
                input,
                output,
                vec![Data::Simulation, Data::Task(*task_id)],
                0.0,
                vec![],
            );

            net.transitions.insert(transition.id, transition);
        }

        // Add all the primitives to map to the name of the primitive's type
        for (primitive_id, primitive) in self.primitives.iter() {
            net.name_lookup
                .insert(*primitive_id, format!("{:?}", primitive.tag()));
        }

        net
    }
}
