use std::io::{Result, Write};

use super::{function, general, trait_impls};
use crate::{
    analysis::{self, special_functions::Type},
    env::Env,
    library,
};

pub fn generate(w: &mut Write, env: &Env, analysis: &analysis::record::Info) -> Result<()> {
    let type_ = analysis.type_(&env.library);

    general::start_comments(w, &env.config)?;
    general::uses(w, env, &analysis.imports)?;

    if analysis.use_boxed_functions {
        if let Some(ref glib_get_type) = analysis.glib_get_type {
            general::define_auto_boxed_type(
                w,
                env,
                &analysis.name,
                &type_.c_type,
                glib_get_type,
                &analysis.derives,
            )?;
        } else {
            panic!(
                "Record {} has record_boxed=true but don't have glib:get_type function",
                analysis.name
            );
        }
    } else if let (Some(ref_fn), Some(unref_fn)) = (
        analysis.specials.get(&Type::Ref),
        analysis.specials.get(&Type::Unref),
    ) {
        general::define_shared_type(
            w,
            env,
            &analysis.name,
            &type_.c_type,
            ref_fn,
            unref_fn,
            &analysis.glib_get_type,
            &analysis.derives,
        )?;
    } else if let (Some(copy_fn), Some(free_fn)) = (
        analysis.specials.get(&Type::Copy),
        analysis.specials.get(&Type::Free),
    ) {
        general::define_boxed_type(
            w,
            env,
            &analysis.name,
            &type_.c_type,
            copy_fn,
            free_fn,
            &analysis.glib_get_type,
            &analysis.derives,
        )?;
    } else if let Some(ref glib_get_type) = analysis.glib_get_type {
        general::define_auto_boxed_type(
            w,
            env,
            &analysis.name,
            &type_.c_type,
            glib_get_type,
            &analysis.derives,
        )?;
    } else {
        panic!(
            "Missing memory management functions for {}",
            analysis.full_name
        );
    }

    if analysis.functions.iter().any(|f| !f.visibility.hidden()) {
        writeln!(w)?;
        write!(w, "impl {} {{", analysis.name)?;

        for func_analysis in &analysis.functions {
            function::generate(w, env, func_analysis, false, false, 1)?;
        }

        writeln!(w, "}}")?;
    }

    general::declare_default_from_new(w, env, &analysis.name, &analysis.functions)?;

    trait_impls::generate(
        w,
        &analysis.name,
        &analysis.functions,
        &analysis.specials,
        None,
    )?;

    if analysis.concurrency != library::Concurrency::None {
        writeln!(w)?;
    }

    match analysis.concurrency {
        library::Concurrency::Send | library::Concurrency::SendSync => {
            writeln!(w, "unsafe impl Send for {} {{}}", analysis.name)?;
        }
        library::Concurrency::SendUnique => {
            panic!("SendUnique concurrency can only be autogenerated for GObject subclasses");
        }
        _ => (),
    }

    if analysis.concurrency == library::Concurrency::SendSync {
        writeln!(w, "unsafe impl Sync for {} {{}}", analysis.name)?;
    }

    Ok(())
}

pub fn generate_reexports(
    env: &Env,
    analysis: &analysis::record::Info,
    module_name: &str,
    contents: &mut Vec<String>,
) {
    let cfg_condition = general::cfg_condition_string(&analysis.cfg_condition, false, 0);
    let version_cfg = general::version_condition_string(env, analysis.version, false, 0);
    let mut cfg = String::new();
    if let Some(s) = cfg_condition {
        cfg.push_str(&s);
        cfg.push('\n');
    };
    if let Some(s) = version_cfg {
        cfg.push_str(&s);
        cfg.push('\n');
    };
    contents.push("".to_owned());
    contents.push(format!("{}mod {};", cfg, module_name));
    contents.push(format!(
        "{}pub use self::{}::{};",
        cfg, module_name, analysis.name
    ));
}
