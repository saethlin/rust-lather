extern crate rgsl;
use self::rgsl::types::multifit_solver::MultiFitFdfSolverType;

use std::rc::Rc;
use std::cell::RefCell;

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
        f.set(i, fit-y);
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

pub fn fit_rv(rv: &Vec<f64>, ccf: &Vec<f64>, guess: &Gaussian) -> Gaussian {
    let mut solver = rgsl::MultiFitFdfSolver::new(&MultiFitFdfSolverType::lmsder(), rv.len(), 4)
        .unwrap();

    let data = Rc::new(RefCell::new(Data {
        x: rv.clone(),
        y: ccf.clone(),
    }));
    let mut x_init: [f64; 4] = [guess.height, guess.centroid, guess.width, guess.offset];
    let mut x = rgsl::VectorView::from_array(&mut x_init);

    let mut solver_func = rgsl::MultiFitFunctionFdf::new(rv.len(), 4);

    /*
    let gaussb_f =
        clone!(
            data => move |x, solver_func| {
                gauss_f(&x, &mut solver_func, &*data.borrow())
            }
        );*/
    let gaussb_f = {
        let data = data.clone();
        move |x, mut solver_func| gauss_f(&x, &mut solver_func, &*data.borrow())
    };
    solver_func.f = Some(Box::new(gaussb_f));

    /*
    let gaussb_df =
        clone!(data => move |x, jacobian| {
        gauss_df(&x, &mut jacobian, &*data.borrow())
    });*/
    let gaussb_df = {
        let data = data.clone();
        move |x, mut jacobian| gauss_df(&x, &mut jacobian, &*data.borrow())
    };
    solver_func.df = Some(Box::new(gaussb_df));

    /*
    let gaussb_fdf =
        clone!(data => move |x, solver_func, jacobian| {
        gauss_f(&x, &mut solver_func, &*data.borrow());
        gauss_df(&x, &mut jacobian, &*data.borrow());
        rgsl::Value::Success
    });
    */
    let gaussb_fdf = {
        let data = data.clone();
        move |x, mut solver_func, mut jacobian| {
            gauss_f(&x, &mut solver_func, &*data.borrow());
            gauss_df(&x, &mut jacobian, &*data.borrow());
            rgsl::Value::Success
        }
    };
    solver_func.fdf = Some(Box::new(gaussb_fdf));

    solver.set(&mut solver_func, &x.vector());

    for _ in 0..100 {
        if solver.iterate() != rgsl::Value::Success {
            break;
        }
        if rgsl::multifit::test_delta(&solver.dx(), &solver.x(), 0.0, 1e-10) != rgsl::Value::Continue {
            break;
        }
    }

     Gaussian {
        height: solver.x().get(0),
        centroid: solver.x().get(1),
        width: solver.x().get(2),
        offset: solver.x().get(3),
    }
}
