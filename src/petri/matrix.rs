use nalgebra::base::DMatrix;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatrixNet {
    pub id: Uuid,
    pub name: String,
    // F in most PetriNet representations
    // Presence/absence of an arc between (places and transitions) and (transitions and places)
    pub arcs: DMatrix<i64>,
    // V in most PetriNet representations
    // Weight (multiplicity) of the arcs
    pub weights: DMatrix<i64>,
    // C in most PetriNet representations
    // Incidence is the change in each place for a given transition
    pub incidence: DMatrix<i64>,
    pub marking: DMatrix<i64>
}