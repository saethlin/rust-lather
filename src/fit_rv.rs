extern crate rgsl;
extern crate gnuplot;
use self::gnuplot::{Figure, AxesCommon, Color};
use std::borrow::Borrow;

use self::rgsl::types::multifit_solver::MultiFitFdfSolverType;

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( mut $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

#[derive(Clone)]
pub struct Data {
    x: Vec<f64>,
    y: Vec<f64>,
}

#[derive(Clone)]
pub struct Gaussian {
    pub height: f64,
    pub centroid: f64,
    pub width: f64,
    pub offset: f64,
}

fn gauss_f(x: &rgsl::VectorF64, f: &mut rgsl::VectorF64, data: &Data) -> rgsl::Value {
    let exp = f64::exp;

    let height = x.get(0);
    let centroid = x.get(1);
    let width = x.get(2);
    let offset = x.get(3);

    for (i, (x, y)) in data.x.iter().zip(data.y.iter()).enumerate() {
        let fit = height * exp(-(x - centroid) * (x - centroid) / (2.0 * width * width)) + offset;
        f.set(i, y-fit);
    }

    rgsl::Value::Success
}

fn gauss_df(x: &rgsl::VectorF64, jacobian: &mut rgsl::MatrixF64, data: &Data) -> rgsl::Value {
    let exp = f64::exp;

    let height = x.get(0);
    let centroid = x.get(1);
    let width = x.get(2);

    for (i, x) in data.x.iter().enumerate() {
        let tmp = height * exp(-(x - centroid) * (x - centroid) / (2.0 * width * width));
        jacobian.set(i, 0, tmp / height);
        jacobian.set(i, 1, tmp * (x - centroid) / (width * width));
        jacobian.set(i, 2, tmp * (x - centroid) * (x - centroid) / (width * width * width));
        jacobian.set(i, 3, 1.0);
    }

    rgsl::Value::Success
}

fn print_state(iter: usize, s: &rgsl::MultiFitFdfSolver) {
    println!("iter: {} x = {} {} {} {} |f(x)| = {}", iter,
             s.x().get(0), s.x().get(1), s.x().get(2), s.x().get(3), rgsl::blas::level1::dnrm2(&s.f()));
}

#[allow(unused_mut)]
pub fn fit_rv(rv: &Vec<f64>, ccf: &Vec<f64>, guess: &Gaussian) -> Gaussian {
    let mut solver_func = rgsl::MultiFitFunctionFdf::new(rv.len(), 4);
    let mut solver = rgsl::MultiFitFdfSolver::new(&MultiFitFdfSolverType::lmsder(), rv.len(), 4)
        .unwrap();
    solver.x().set(0, guess.height);
    solver.x().set(1, guess.centroid);
    solver.x().set(2, guess.width);
    solver.x().set(3, guess.offset);

    // TODO remove these clones?
    let data = Data {
        x: rv.clone(),
        y: ccf.clone(),
    };

    let gaussb_f =
        clone!(data => move |x, f| {
        gauss_f(&x, &mut f, &*data.borrow())
    });
    solver_func.f = Some(Box::new(gaussb_f));


    let gaussb_df =
        clone!(data => move |x, jacobian| {
        gauss_df(&x, &mut jacobian, &*data.borrow())
    });
    solver_func.df = Some(Box::new(gaussb_df));

    let gaussb_fdf =
        clone!(data => move |x, f, jacobian| {
        gauss_f(&x, &mut f, &*data.borrow());
        gauss_df(&x, &mut jacobian, &*data.borrow());
        rgsl::Value::Success
    });
    solver_func.fdf = Some(Box::new(gaussb_fdf));

    let mut status;
    for i in 0..100 {
        status = solver.iterate();

        println!("status = {}", rgsl::error::str_error(status));
        print_state(i, &solver);

        if status != rgsl::Value::Success {
            break;
        }
        status = rgsl::multifit::test_delta(&solver.dx(), &solver.x(), 0.0, 1e-5);
        if status != rgsl::Value::Continue {
            break;
        }
    }
    /*
    let guess_func = rv.iter().map(|x|
        guess.height * f64::exp(-(x - guess.centroid) * (x - guess.centroid) / (2.0 * guess.width * guess.width)) + guess.offset)
        .collect::<Vec<f64>>();

    let mut rv_fig = Figure::new();
    rv_fig.axes2d()
        .lines(&data.x, ccf, &[Color("black")])
        .lines(&data.x, &guess_func, &[Color("red")]);
    rv_fig.show();
    */

     Gaussian {
        height: solver.x().get(0),
        centroid: solver.x().get(1),
        width: solver.x().get(2),
        offset: solver.x().get(3),
    }
}
