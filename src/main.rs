use dioxus::core_macro::render;
use dioxus::hooks::{use_state, use_effect};
use dioxus::core::{VirtualDom, Element};
use dioxus_core::Scope;
use futures_util::task::ArcWake;
use futures_util::{pin_mut, FutureExt};
use gl::types::GLint;
use glutin::GlProfile;
use glutin::event::{Event, StartCause};
use skia_safe::{Surface, ColorType};
use skia_safe::gpu::{BackendRenderTarget, SurfaceOrigin};
use skia_safe::gpu::gl::FramebufferInfo;
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


fn main() {

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let _guard = rt.enter();

    let rdom = Arc::new(Mutex::new(RealDom::<NodeState>::new()));
    let mut dom = VirtualDom::new(app);

    let muts = dom.rebuild();
    let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(muts);

    let ctx = SendAnyMap::new();
    rdom.lock().unwrap().update_state(to_update, ctx);

    let event_loop = EventLoop::<()>::with_user_event();

    let proxy = event_loop.create_proxy();

    let waker = tao_waker(&proxy);

    let wb = WindowBuilder::new()
        .with_title("test");

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core);

    let windowed_context = cb.build_windowed(wb, &event_loop).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: skia_safe::gpu::gl::Format::RGBA8.into(),
        }
    };

    let mut gr_context = skia_safe::gpu::DirectContext::new_gl(None, None).unwrap();

    let mut surface = create_surface(&windowed_context, &fb_info, &mut gr_context);
    let sf = windowed_context.window().scale_factor() as f32;
    surface.canvas().scale((sf, sf));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("NOW");
                _ = proxy.send_event(());
            },
            Event::UserEvent(_) => {
                println!("polling");
                poll_vdom(&waker, &mut dom, &rdom);
            },
            Event::WindowEvent { .. } => {
                windowed_context.window().request_redraw();
            }
            Event::LoopDestroyed => {}
            Event::RedrawRequested(_) => {
                gr_context.flush(None);
                windowed_context.swap_buffers().unwrap();
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

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;

pub fn create_surface(
    windowed_context: &WindowedContext,
    fb_info: &FramebufferInfo,
    gr_context: &mut skia_safe::gpu::DirectContext,
) -> Surface {
    let pixel_format = windowed_context.get_pixel_format();
    let size = windowed_context.window().inner_size();
    let backend_render_target = BackendRenderTarget::new_gl(
        (
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        ),
        pixel_format.multisampling.map(|s| s.try_into().unwrap()),
        pixel_format.stencil_bits.try_into().unwrap(),
        *fb_info,
    );
    Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap()
}