use crate::core::panoc::*;
use crate::core::*;
use crate::mocks;
use std::num::NonZeroUsize;

const N_DIM: usize = 2;
#[test]
fn t_panoc_init() {
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
        assert!(2.549509967743775 > panoc_engine.cache.lipschitz_constant);
        assert!(0.372620625931781 < panoc_engine.cache.gamma, "gamma");
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
    }
    println!("cache = {:#?}", &panoc_cache);
}

fn print_panoc_engine<'a, GradientType, ConstraintType, CostType>(
    panoc_engine: &PANOCEngine<'a, GradientType, ConstraintType, CostType>,
) where
    GradientType: Fn(&[f64], &mut [f64]) -> i32,
    CostType: Fn(&[f64], &mut f64) -> i32,
    ConstraintType: constraints::Constraint,
{
    println!("> fpr       = {:?}", &panoc_engine.cache.gamma_fpr);
    println!("> fpr       = {:.2e}", panoc_engine.cache.norm_gamma_fpr);
    println!("> L         = {:.3}", panoc_engine.cache.lipschitz_constant);
    println!("> gamma     = {:.10}", panoc_engine.cache.gamma);
    println!("> tau       = {:.3}", panoc_engine.cache.tau);
    println!("> lbfgs dir = {:.11?}", panoc_engine.cache.direction_lbfgs);
}

#[test]
fn t_test_panoc_basic() {
    let bounds = constraints::Ball2::new_at_origin_with_radius(0.2);
    let problem = Problem::new(bounds, mocks::my_gradient, mocks::my_cost);
    let tolerance = 1e-9;
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(2).unwrap(),
        tolerance,
        NonZeroUsize::new(5).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);

    let mut u = [0.0, 0.0];
    panoc_engine.init(&mut u);
    panoc_engine.step(&mut u);
    let fpr0 = panoc_engine.cache.norm_gamma_fpr;
    println!("fpr0 = {}", fpr0);

    for i in 1..=100 {
        println!("----------------------------------------------------");
        println!("> iter      = {}", i);
        print_panoc_engine(&panoc_engine);
        println!("> u         = {:.14?}", u);
        if !panoc_engine.step(&mut u) {
            break;
        }
    }
    println!("final |fpr| = {}", panoc_engine.cache.norm_gamma_fpr);
    assert!(panoc_engine.cache.norm_gamma_fpr <= tolerance);
    unit_test_utils::assert_nearly_equal_array(&u, &mocks::SOLUTION_A, 1e-6, 1e-8, "");
}

#[test]
fn t_test_panoc_hard() {
    let radius: f64 = 0.05;
    let bounds = constraints::Ball2::new_at_origin_with_radius(radius);
    let problem = Problem::new(
        bounds,
        mocks::hard_quadratic_gradient,
        mocks::hard_quadratic_cost,
    );
    let n: usize = 3;
    let lbfgs_memory: usize = 10;
    let tolerance_fpr: f64 = 1e-12;
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(n).unwrap(),
        tolerance_fpr,
        NonZeroUsize::new(lbfgs_memory).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);

    let mut u = [-20., 10., 0.2];
    panoc_engine.init(&mut u);

    println!("L     = {}", panoc_engine.cache.lipschitz_constant);
    println!("gamma = {}", panoc_engine.cache.gamma);
    println!("sigma = {}", panoc_engine.cache.sigma);

    let mut i = 1;
    println!("\n*** ITERATION   1");
    while panoc_engine.step(&mut u) && i < 100 {
        i += 1;
        println!("+ u_plus               = {:?}", u);
        println!("\n*** ITERATION {:3}", i);
    }

    println!("\nsol = {:?}", u);
    assert!(panoc_engine.cache.norm_gamma_fpr <= tolerance_fpr);
    unit_test_utils::assert_nearly_equal_array(&u, &mocks::SOLUTION_HARD, 1e-6, 1e-8, "");
}

#[test]
fn t_test_panoc_rosenbrock() {
    let tolerance = 1e-12;
    let a = 1.0;
    let b = 100.0;
    let df = |u: &[f64], grad: &mut [f64]| -> i32 {
        mocks::rosenbrock_grad(a, b, u, grad);
        0
    };
    let f = |u: &[f64], c: &mut f64| -> i32 {
        *c = mocks::rosenbrock_cost(a, b, u);
        0
    };
    let bounds = constraints::Ball2::new_at_origin_with_radius(1.0);
    let problem = Problem::new(bounds, df, f);
    let mut panoc_cache = PANOCCache::new(
        NonZeroUsize::new(2).unwrap(),
        tolerance,
        NonZeroUsize::new(2).unwrap(),
    );
    let mut panoc_engine = PANOCEngine::new(problem, &mut panoc_cache);
    let mut u = [-1.5, 0.9];
    panoc_engine.init(&mut u);
    let mut i = 1;
    while panoc_engine.step(&mut u) && i < 50 {
        i += 1;
    }
    assert!(panoc_engine.cache.norm_gamma_fpr <= tolerance);
    println!("u = {:?}", u);
}
