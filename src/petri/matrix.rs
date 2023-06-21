use nalgebra::base::DMatrix;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct MatrixNet {
    pub id: Uuid,
    pub name: String,
    pub f: DMatrix<u128>,
    pub v: DMatrix<u128>,
    pub c: DMatrix<i128>,
    pub marking: DMatrix<u128>
}