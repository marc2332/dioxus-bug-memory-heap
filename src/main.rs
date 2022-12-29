use dioxus::core_macro::render;
use dioxus::hooks::{use_state, use_effect};
use dioxus::core::{VirtualDom, Element};
use dioxus_core::Scope;
use glutin::event::Event;
use tokio::time::sleep;
use dioxus_native_core::{SendAnyMap, real_dom::RealDom, state::{State, ParentDepState}, node_ref::{NodeView, AttributeMask}, NodeMask};
use dioxus_native_core_macro::{sorted_str_slice, State};
use gl::types::*;
use glutin::event_loop::{EventLoop, ControlFlow};
use glutin::window::WindowId;
use glutin::{window::WindowBuilder, GlProfile};
use skia_safe::{
    gpu::{gl::FramebufferInfo, BackendRenderTarget, SurfaceOrigin},
    ColorType, Surface,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration},
};

type WindowedContext = glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::window::Window>;


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
    std::thread::spawn(move || {
        let rdom = Arc::new(Mutex::new(RealDom::<NodeState>::new()));
        let mut dom = VirtualDom::new(app);

        let muts = dom.rebuild();
        let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(muts);

        let ctx = SendAnyMap::new();
        rdom.lock().unwrap().update_state(to_update, ctx);

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                    loop {
                        dom.wait_for_work().await;

                        let mutations = dom.render_immediate();
                        let (to_update, _diff) = rdom.lock().unwrap().apply_mutations(mutations);

                        let ctx = SendAnyMap::new();
                        rdom.lock().unwrap().update_state(to_update, ctx);
                    }
            });
    });

    let event_loop = EventLoop::<WindowId>::with_user_event();

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
        *control_flow = ControlFlow::Poll;

        match event {
            Event::MainEventsCleared => {
                windowed_context.window().request_redraw();
            }
            Event::LoopDestroyed => {}
            Event::WindowEvent { .. } => {
                windowed_context.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                println!("drawing");
                gr_context.flush(None);
                windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

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

    println!("mounted");
    
    render!(
        blabla { }
    )
}
