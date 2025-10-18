use bevy::{
    prelude::*,
    render::{
        RenderApp,
        graph::CameraDriverLabel,
        render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{
            BlendComponent, BlendState, ColorTargetState, ColorWrites, LoadOp, Operations,
            RawFragmentState, RawRenderPipelineDescriptor, RawVertexState,
            RenderPassColorAttachment, RenderPassDescriptor, ShaderModuleDescriptor, ShaderSource,
            StoreOp,
        },
        renderer::RenderContext,
        view::ExtractedWindows,
    },
};

fn main() {
    // Create a new bevy app
    let mut app = App::new();

    // Add the default plugins, which notably includes the RenderPlugin and the WindowPlugin.
    // The RenderPlugin sets up the RenderApp, the render world, and other core rendering features.
    // The WindowPlugin will handle managing windows and add resources to the render world to reference those windows.
    app.add_plugins(DefaultPlugins);

    // Get a reference to the RenderApp sub app so we can make modifications
    let render_app = app.get_sub_app_mut(RenderApp).expect(
        "Since we're using the DefaultPlugins which includes the RenderPlugin, /
        we should be able to get the render app.",
    );

    // Grab a reference to the RenderGraph from the render world
    let mut render_graph = render_app
        .world_mut()
        .get_resource_mut::<RenderGraph>()
        .expect(
            "Since we're using the DefaultPlugins which includes the RenderPlugin, /
            we should be able to get the render graph.",
        );

    // Remove bevy's built in render node. We'll cover this later.
    _ = render_graph.remove_node(CameraDriverLabel);

    // Add in our own node to the render graph. It'll now run every frame when the render graph is run.
    render_graph.add_node(WebGPUTriangleNodeLabel, WebGPUTriangleNode);

    // Start the app!
    app.run();
}

// RenderLabels are how bevy identifies nodes in the render graph.
// We'll use this one for our own custom node.
#[derive(Debug, Hash, Eq, PartialEq, Clone, RenderLabel)]
struct WebGPUTriangleNodeLabel;

// We'll define a struct to make our Node. We could store node state in this struct,
//  but our node is too simple to need any state.
#[derive(Default)]
struct WebGPUTriangleNode;

impl render_graph::Node for WebGPUTriangleNode {
    fn input(&self) -> Vec<render_graph::SlotInfo> {
        Vec::new()
    }

    fn output(&self) -> Vec<render_graph::SlotInfo> {
        Vec::new()
    }

    fn update(&mut self, _world: &mut World) {}

    fn run<'w>(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        // Grab the resource that contains all the windows the app currently has.
        let windows = world.resource::<ExtractedWindows>();

        // Let's ignore other windows for now, and just grab the primary window.
        // It's possible there is no primary window, which is likely to happen when a user
        //  closes the app's window, but the app hasn't shut down yet.
        // If we ever can't get the primary window, we'll just return early and do nothing.
        let primary_window = match windows.get(match &windows.primary {
            Some(window_entity) => window_entity,
            None => return Ok(()),
        }) {
            Some(primary_window) => primary_window,
            None => return Ok(()),
        };

        // Create and validate a shader module from out webgpu_triangle.wesl file.
        // Note that the include_str! macro will copy the contents of the file into an &str
        //  at compile time to include in the binary.
        // We'll worry about properly loading shaders from files later.
        let shader_module = render_context
            .render_device()
            .create_and_validate_shader_module(ShaderModuleDescriptor {
                label: Some("WebGPU Triangle Shader"),
                source: ShaderSource::Wgsl(
                    include_str!("../assets/shader/webgpu_triangle.wesl").into(),
                ),
            });

        // Create a RawRenderPipelineDescriptor which tells the GPU what shaders we want to run,
        //  as well as what inputs they have, what texture they should output to, and more.
        let pipeline =
            render_context
                .render_device()
                .create_render_pipeline(&RawRenderPipelineDescriptor {
                    label: Some("Hardcoded WebGPU Triangle Pipeline"),
                    layout: None,
                    vertex: RawVertexState {
                        module: &shader_module,
                        entry_point: None,
                        compilation_options: default(),
                        buffers: &[],
                    },
                    primitive: default(),
                    depth_stencil: None,
                    multisample: default(),
                    fragment: Some(RawFragmentState {
                        module: &shader_module,
                        entry_point: None,
                        compilation_options: default(),
                        targets: &[Some(ColorTargetState {
                            format: primary_window
                                .swap_chain_texture_format
                                .expect("swap chain texture format should exist"),
                            blend: Some(BlendState {
                                color: BlendComponent::REPLACE,
                                alpha: BlendComponent::REPLACE,
                            }),
                            write_mask: ColorWrites::all(),
                        })],
                    }),
                    multiview: None,
                    cache: None,
                });

        // Create a RenderPassDescriptor, which gives the GPU a list of textures to write to.
        // In our case, we're only outputing color to the window's swap chain texture.
        let render_pass_descriptor = RenderPassDescriptor {
            label: Some("WebGPU Triangle Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &primary_window
                    .swap_chain_texture_view
                    .as_ref()
                    .expect("There should be a swap chain texture"),
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::WHITE.to_linear().into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        // Begin the Render pass,
        let mut render_pass = render_context
            .command_encoder()
            .begin_render_pass(&render_pass_descriptor);

        // Set the pipeline,
        render_pass.set_pipeline(&pipeline);

        // And tell the GPU to draw verticies 0, 1, and 2 one time.
        render_pass.draw(0..3, 0..1);

        // Now that our commands are queued, that's all we need to do.
        // When the render graph is done running, it will automatically collect
        //  and submit all the queued render commands to the GPU.
        Ok(())
    }
}
