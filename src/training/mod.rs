pub mod data;
pub mod loss;
pub mod optimizer;

pub use data::{TrainingData, SyntheticData, ArrayData};
pub use loss::{LossFunction, MSE, CrossEntropy, MAE};
pub use optimizer::{Optimizer, SGD};

