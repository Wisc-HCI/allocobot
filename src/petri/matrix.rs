use nalgebra::base::DMatrix;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct MatrixNet {
    pub id: Uuid,
    pub name: String,
    pub F: DMatrix<u128>,
    pub V: DMatrix<u128>,
    pub C: DMatrix<i128>,
    pub marking: DMatrix<u128>
}