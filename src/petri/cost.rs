use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::vec::Vec;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CostFrequency {
    Once,
    Extrapolated,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CostCategory {
    Ergonomic,
    Monetary,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cost {
    pub frequency: CostFrequency,
    pub value: f64,
    pub category: CostCategory,
}

pub type CostSet = Vec<Cost>;

pub fn add_cost_sets(a: &CostSet, b: &CostSet) -> CostSet {
    let mut result = vec![];
    for freq in vec![CostFrequency::Once, CostFrequency::Extrapolated].iter() {
        for cat in vec![CostCategory::Ergonomic, CostCategory::Monetary].iter() {
            let mut sum = 0.0;
            for cost in a
                .iter()
                .filter(|c| c.frequency == *freq && c.category == *cat)
            {
                sum += cost.value;
            }
            for cost in b
                .iter()
                .filter(|c| c.frequency == *freq && c.category == *cat)
            {
                sum += cost.value;
            }
            if sum > 0.0 {
                result.push(Cost {
                    frequency: freq.clone(),
                    value: sum,
                    category: cat.clone(),
                });
            }
        }
    }
    return result;
}
