use dioxus::core_macro::render;
use dioxus::hooks::{use_state, use_effect};
use dioxus::core::{VirtualDom, Element};
use dioxus_core::Scope;
use futures_util::task::ArcWake;
use futures_util::{pin_mut, FutureExt};
use glutin::event::{Event, StartCause};
use tokio::time::sleep;
use dioxus_native_core::{SendAnyMap, real_dom::RealDom, state::{State, ParentDepState}, node_ref::{NodeView, AttributeMask}, NodeMask};
use dioxus_native_core_macro::{sorted_str_slice, State};
use glutin::event_loop::{EventLoop, ControlFlow, EventLoopProxy};
use glutin::{window::WindowBuilder};
use std::task::Waker;
use std::{
    sync::{Arc, Mutex},
    time::{Duration},
};

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

    let event_loop = EventLoop::<()>::with_user_event();

    let window = WindowBuilder::new()
        .with_title("test")
        .build(&event_loop)
        .unwrap();

    let proxy = event_loop.create_proxy();

    let waker = tao_waker(&proxy);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("NOW");
                _ = proxy.send_event(());
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            },
            Event::UserEvent(_) => {
                println!("polling");
                poll_vdom(&waker, &mut dom, &rdom);
            },
            Event::LoopDestroyed => {}
            Event::RedrawRequested(_) => {
              
            }
            _ => (),
        }
    });
}

    
fn poll_vdom(waker: &Waker, dom: &mut VirtualDom, rdom: &Arc<Mutex<RealDom<NodeState>>>) {
    let mut cx = std::task::Context::from_waker(waker);

    loop {
        {
            let fut = dom.wait_for_work();
            pin_mut!(fut);

            match fut.poll_unpin(&mut cx) {
                std::task::Poll::Ready(_) => {}
                std::task::Poll::Pending => break,
            }
        }

        let mutations = dom.render_immediate();
        let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(mutations);

        let ctx = SendAnyMap::new();
        rdom.lock().unwrap().update_state(to_update, ctx);
    }

    println!("stopped");
}


pub fn tao_waker(proxy: &EventLoopProxy<()>) -> std::task::Waker {
    struct DomHandle(EventLoopProxy<()>);

    // this should be implemented by most platforms, but ios is missing this until
    // https://github.com/tauri-apps/wry/issues/830 is resolved
    unsafe impl Send for DomHandle {}
    unsafe impl Sync for DomHandle {}

    impl ArcWake for DomHandle {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            _ = arc_self.0.send_event(());
        }
    }

    futures_util::task::waker(Arc::new(DomHandle(proxy.clone())))
}

fn app(cx: Scope) -> Element {
    let padding = use_state(&cx, || 10);

    use_effect(&cx, padding, |padding| async move {
        sleep(Duration::from_millis(100)).await;
        padding.with_mut(|padding| {
            if *padding < 2000 {
                *padding += 1;
            } else {
                *padding = 0;
            }
        });
    });

    println!("mounted");
    
    render!(
        blabla { }
    )
}
