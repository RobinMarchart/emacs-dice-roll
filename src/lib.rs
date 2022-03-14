use std::{sync::Arc, thread};

use anyhow::anyhow;
use dice_roll::{dice_roll::ExpressionEvaluate, dice_roll::TermEvaluate, Expression, Term};
use emacs::{defun, Env, IntoLisp, Transfer, Value};
use emacs_native_async::{to_lisp::ToLispConvert, NotificationHandler};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_xoshiro::Xoshiro256PlusPlus;

struct Evaluator{
    rng:ChaCha20Rng
}

impl Transfer for Evaluator {}

impl Evaluator {
    fn new() -> Self {
        Evaluator {
            rng: ChaCha20Rng::from_entropy(),
        }
    }
    fn seed(&mut self) -> Xoshiro256PlusPlus {
        let mut seed: <Xoshiro256PlusPlus as SeedableRng>::Seed = Default::default();
        self.rng.fill(&mut seed);
        Xoshiro256PlusPlus::from_seed(seed)
    }
}

fn expr_res_to_value(env: &Env, value: Vec<(i64, Vec<i64>)>) -> emacs::Result<Value> {
    let res = env.make_vector(value.len(), 0_i64)?;
    for (index, term) in value.into_iter().enumerate() {
        res.set(index, term_res_to_value(env, term)?)?
    }
    res.into_lisp(env)
}

fn term_res_to_value(env: &Env, value: (i64, Vec<i64>)) -> emacs::Result<Value> {
    let thr = env.make_vector(value.1.len(), 0_i64)?;
    for (index, r) in value.1.into_iter().enumerate() {
        thr.set(index, r)?
    }
    env.vector((value.0, thr))
}

#[allow(unused)]
fn main() {
    emacs::plugin_is_GPL_compatible!();

    #[emacs::module(name = "dice-roll-impl", separator = "-")]
    fn init(env: &Env) -> emacs::Result<()> {
        #[defun]
        fn init(env: &Env) -> emacs::Result<Box<Evaluator>> {
            Ok(Box::new(Evaluator::new()))
        }

        #[defun]
        fn parse_expr(env: &Env, expr: String) -> emacs::Result<Arc<Expression>> {
            Ok(Arc::new(
                dice_roll::parser::parse_expression(&expr)
                    .map_err(|_| anyhow!("unable to parse expr"))?
                    .1,
            ))
        }
        #[defun]
        fn parse_term(env: &Env, expr: String) -> emacs::Result<Arc<Term>> {
            Ok(Arc::new(
                dice_roll::parser::parse_term(&expr)
                    .map_err(|_| anyhow!("unable to parse term"))?
                    .1,
            ))
        }

        #[defun]
        fn roll_expr<'e>(
            env: &'e Env,
            evaluator: &mut Evaluator,
            expr: &Arc<Expression>,
        ) -> emacs::Result<Value<'e>> {
            expr_res_to_value(env, expr.evaluate(&mut || true, &mut evaluator.seed())?)
        }
        #[defun]
        fn roll_term<'e>(
            env: &'e Env,
            evaluator: &mut Evaluator,
            term: &Arc<Term>,
        ) -> emacs::Result<Value<'e>> {
            term_res_to_value(env, term.evaluate(&mut || true, &mut evaluator.seed())?)
        }

        #[defun]
        fn roll_expr_async(
            env: &Env,
            evaluator: &mut Evaluator,
            expr: &Arc<Expression>,
            notifications: &Arc<NotificationHandler>,
        ) -> emacs::Result<i64> {
            let mut gen = evaluator.seed();
            let id = notifications.register();
            let notifications = notifications.clone();
            let expr = expr.clone();
            thread::spawn(move ||{
                let res=expr.evaluate(&mut||true, &mut gen);
                notifications.submit(Ok(ToLispConvert::lazy(move|env|expr_res_to_value(env, res?))), id)
            });
            Ok(id)
        }

        #[defun]
        fn roll_term_async(
            env: &Env,
            evaluator: &mut Evaluator,
            term: &Arc<Term>,
            notifications: &Arc<NotificationHandler>,
        ) -> emacs::Result<i64> {
            let mut gen = evaluator.seed();
            let id = notifications.register();
            let notifications = notifications.clone();
            let term = term.clone();
            thread::spawn(move ||{
                let res=term.evaluate(&mut||true, &mut gen);
                notifications.submit(Ok(ToLispConvert::lazy(move|env|term_res_to_value(env, res?))), id)
            });
            Ok(id)
        }


        Ok(())
    }
}
