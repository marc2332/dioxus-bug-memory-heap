#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dioxus::core_macro::render;
use dioxus::hooks::{use_state, use_effect};
use dioxus::core::{VirtualDom, Element};
use dioxus_core::Scope;
use std::{time::Duration, sync::{Arc, Mutex}};
use tokio::time::sleep;
use dioxus_native_core::{SendAnyMap, real_dom::RealDom, state::{State, ParentDepState}, node_ref::{NodeView, AttributeMask}, NodeMask};
use dioxus_native_core_macro::{sorted_str_slice, State};


#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlablaState {
    
}


/// Font style are inherited by default if not specified otherwise by some of the supported attributes.
impl ParentDepState for BlablaState {
    type Ctx = ();
    type DepState = (Self,);

    const NODE_MASK: NodeMask =
        NodeMask::new_with_attrs(AttributeMask::Static(&sorted_str_slice!([
            "blabla",
        ])));

    fn reduce<'a>(
        &mut self,
        _node: NodeView,
        _parent: Option<(&'a Self,)>,
        _ctx: &Self::Ctx,
    ) -> bool {
        false
    }
}

#[derive(Clone, State, Default, Debug)]
pub struct NodeState {
    #[parent_dep_state(blabla)]
    blabla: BlablaState
}

mod dioxus_elements {
    macro_rules! builder_constructors {
        (
            $(
                $(#[$attr:meta])*
                $name:ident {
                    $(
                        $(#[$attr_method:meta])*
                        $fil:ident: $vil:ident,
                    )*
                };
            )*
        ) => {
            $(
                #[allow(non_camel_case_types)]
                $(#[$attr])*
                pub struct $name;

                impl $name {
                    pub const TAG_NAME: &'static str = stringify!($name);
                    pub const NAME_SPACE: Option<&'static str> = None;

                    $(
                        pub const $fil: (&'static str, Option<&'static str>, bool) = (stringify!($fil), None, false);
                    )*
                }

                impl GlobalAttributes for $name {}
            )*
        }
    }

    pub trait GlobalAttributes {}

    pub trait SvgAttributes {}
    

    builder_constructors! {
        blabla {

        };
    }
}


#[tokio::main]
async fn main() {
    let rdom = Arc::new(Mutex::new(RealDom::<NodeState>::new()));
    let mut dom = VirtualDom::new(app);

    let muts = dom.rebuild();
    let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(muts);

    let ctx = SendAnyMap::new();
    rdom.lock().unwrap().update_state(to_update, ctx);

    loop {
        dom.wait_for_work().await;

        let mutations = dom.render_immediate();
        let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(mutations);

        let ctx = SendAnyMap::new();
        rdom.lock().unwrap().update_state(to_update, ctx);
    }
}

fn app(cx: Scope) -> Element {
    let padding = use_state(&cx, || 10);

    use_effect(&cx, padding, |padding| async move {
        sleep(Duration::from_millis(1)).await;
        padding.with_mut(|padding| {
            if *padding < 2000 {
                *padding += 1;
            } else {
                *padding = 0;
            }
        });
    });
    
    render!(
        blabla { }
    )
}
