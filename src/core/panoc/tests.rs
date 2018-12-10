use crate::core::panoc::*;
use crate::core::*;
use crate::mocks;
use std::num::NonZeroUsize;

const N_DIM: usize = 2;
#[test]
fn panoc_init() {
    let radius = 0.2;
    let ball = constraints::Ball2::new_at_origin_with_radius(radius);
    let problem = Problem::new(ball, mocks::my_gradient, mocks::my_cost);
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(N_DIM).unwrap(),
        1e-6,
        NonZeroUsize::new(5).unwrap(),
    );

    {
        let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);
        let mut u = [0.75, -1.4];
        panoc_engine.init(&mut u);
        unit_test_utils::assert_nearly_equal(
            2.549509967743775,
            panoc_engine.cache.lipschitz_constant,
            1e-4,
            1e-10,
            "lipschitz",
        );
        unit_test_utils::assert_nearly_equal(
            0.372620625931781,
            panoc_engine.cache.gamma,
            1e-4,
            1e-10,
            "gamma",
        );
        unit_test_utils::assert_nearly_equal(
            0.009129205335329,
            panoc_engine.cache.sigma,
            1e-4,
            1e-10,
            "sigma",
        );
        println!("----------- {} ", panoc_engine.cache.cost_value);
        unit_test_utils::assert_nearly_equal(
            6.34125,
            panoc_engine.cache.cost_value,
            1e-4,
            1e-10,
            "cost value",
        );
        unit_test_utils::assert_nearly_equal_array(
            &[0.35, -3.05],
            &panoc_engine.cache.gradient_u,
            1e-4,
            1e-10,
            "gradient at u",
        );
        unit_test_utils::assert_nearly_equal_array(
            &[0.619582780923877, -0.263507090908068],
            &panoc_engine.cache.gradient_step,
            1e-4,
            1e-10,
            "gradient step",
        );

        unit_test_utils::assert_nearly_equal_array(
            &[0.184046458737518, -0.078274523481010],
            &panoc_engine.cache.u_half_step,
            1e-3,
            1e-8,
            "u_half_step",
        );

        unit_test_utils::assert_nearly_equal_array(&[0.75, -1.4], &u, 1e-4, 1e-9, "u");
    }
    println!("cache = {:#?}", &panoc_cache);
}

fn print_panoc_engine<'a, GradientType, ConstraintType, CostType>(
    panoc_engine: &PANOCEngine<'a, GradientType, ConstraintType, CostType>,
    fpr0: f64,
) where
    GradientType: Fn(&[f64], &mut [f64]) -> i32,
    CostType: Fn(&[f64], &mut f64) -> i32,
    ConstraintType: constraints::Constraint,
{
    println!(
        "> fpr       = {:?}",
        &panoc_engine.cache.fixed_point_residual
    );
    println!("> fpr       = {:.2e}", panoc_engine.cache.norm_fpr);
    println!("> fpr/fpr0  = {:.2e}", panoc_engine.cache.norm_fpr / fpr0);
    println!("> L         = {:.3}", panoc_engine.cache.lipschitz_constant);
    println!("> gamma     = {:.3}", panoc_engine.cache.gamma);
    println!("> tau       = {:.3}", panoc_engine.cache.tau);
    println!("> lbfgs dir = {:.11?}", panoc_engine.cache.direction_lbfgs);
}

#[test]
fn test_panoc_basic() {
    let bounds = constraints::Ball2::new_at_origin_with_radius(0.2);
    let problem = Problem::new(bounds, mocks::my_gradient, mocks::my_cost);
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(2).unwrap(),
        1e-6,
        NonZeroUsize::new(5).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);

    let mut u = [0.0, 0.0];
    panoc_engine.init(&mut u);
    panoc_engine.step(&mut u);
    let fpr0 = panoc_engine.cache.norm_fpr;
    println!("fpr0 = {}", fpr0);

    for i in 1..=100 {
        println!("----------------------------------------------------");
        println!("> iter      = {}", i);
        print_panoc_engine(&panoc_engine, fpr0);
        println!("> u         = {:.14?}", u);
        if !panoc_engine.step(&mut u) {
            break;
        }
    }
    println!("final |fpr| = {}", panoc_engine.cache.norm_fpr);
    assert!(panoc_engine.cache.norm_fpr < 1e-5);
    unit_test_utils::assert_nearly_equal_array(&u, &mocks::SOLUTION, 1e-5, 1e-5, "");
}

#[test]
fn test_panoc_hard() {
    let bounds = constraints::Ball2::new_at_origin_with_radius(1.);
    let problem = Problem::new(
        bounds,
        mocks::hard_quadratic_gradient,
        mocks::hard_quadratic_cost,
    );
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(3).unwrap(),
        1e-5,
        NonZeroUsize::new(10).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);

    let mut u = [0.0, 0.0, 0.0];
    panoc_engine.init(&mut u);
    panoc_engine.cache.lipschitz_constant = 120.0;
    panoc_engine.cache.gamma = 0.95 / 120.;
    panoc_engine.cache.sigma = 1.8e-4_f64;
    panoc_engine.step(&mut u);
    let fpr0 = panoc_engine.cache.norm_fpr;
    println!("fpr0 = {}", fpr0);

    for i in 1..=200 {
        println!("----------------------------------------------------");
        println!("> iter      = {}", i);
        print_panoc_engine(&panoc_engine, fpr0);
        println!("> u         = {:.14?}", u);
        if !panoc_engine.step(&mut u) {
            break;
        }
    }
    println!("final |fpr| = {}", panoc_engine.cache.norm_fpr);
}

#[test]
fn test_panoc_rosenbrock() {
    let df = |u: &[f64], grad: &mut [f64]| -> i32 {
        mocks::rosenbrock_grad(0.1, 10., u, grad);
        0
    };
    let f = |u: &[f64], c: &mut f64| -> i32 {
        *c = mocks::rosenbrock_cost(0.1, 10.0, u);
        0
    };
    let bounds = constraints::Ball2::new_at_origin_with_radius(1.413);
    let problem = Problem::new(bounds, df, f);
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(2).unwrap(),
        1e-6,
        NonZeroUsize::new(5).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);

    let mut u = [1.0, 1.0];

    panoc_engine.init(&mut u);
    panoc_engine.cache.lipschitz_constant = 1e4;

    panoc_engine.step(&mut u);
    println!("> u         = {:.14?}", u);

    panoc_engine.step(&mut u);
    println!("> u         = {:.14?}", u);
}
