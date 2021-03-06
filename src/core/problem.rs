//! An optimization problem
//!
//! This struct defines an optimization problem in terms of its cost function
//! (cost function and its gradient) and constraints
//!
//! Cost functions are user defined. They can either be defined in Rust or in
//! C (and then invoked from Rust via an interface such as icasadi).
//!
use crate::constraints;
pub use crate::core::Problem;

impl<GradientType, ConstraintType, CostType> Problem<GradientType, ConstraintType, CostType>
where
    GradientType: Fn(&[f64], &mut [f64]) -> i32,
    CostType: Fn(&[f64], &mut f64) -> i32,
    ConstraintType: constraints::Constraint,
{
    /// Construct a new instance of an optimisation problem
    ///
    /// ## Arguments
    ///
    /// - `constraints` constraints
    /// - `cost_gradient` gradient of the cost function
    /// - `cost` cost function
    ///
    pub fn new(
        constraints: ConstraintType,
        cost_gradient: GradientType,
        cost: CostType,
    ) -> Problem<GradientType, ConstraintType, CostType> {
        Problem {
            constraints: constraints,
            gradf: cost_gradient,
            cost: cost,
        }
    }
}
