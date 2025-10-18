use bevy::{
    prelude::*,
    render::{
        RenderApp,
        graph::CameraDriverLabel,
        render_graph::{RenderGraph, RenderLabel},
        render_resource::{
            BlendComponent, BlendState, ColorTargetState, ColorWrites, LoadOp, Operations,
            RawFragmentState, RawRenderPipelineDescriptor, RawVertexState,
            RenderPassColorAttachment, RenderPassDescriptor, ShaderModuleDescriptor, ShaderSource,
            StoreOp,
        },
        view::ExtractedWindows,
    },
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    let render_app = app
        .get_sub_app_mut(RenderApp)
        .expect("Unable to get render world");

    let mut render_graph = render_app
        .world_mut()
        .get_resource_mut::<RenderGraph>()
        .expect(
            "RenderGraph not found. Make sure you are using add_render_graph_node on the RenderApp",
        );
    _ = render_graph.remove_node(CameraDriverLabel);
    render_graph.add_node(CustomNodeLabel, WebGPUTriangleNode);
    // render_graph.add_node_edge(CameraDriverLabel, CustomNodeLabel);
    app.run();
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, RenderLabel)]
struct CustomNodeLabel;

#[derive(Default)]
struct WebGPUTriangleNode;

impl bevy::render::render_graph::Node for WebGPUTriangleNode {
    fn input(&self) -> Vec<bevy::render::render_graph::SlotInfo> {
        Vec::new()
    }

    fn output(&self) -> Vec<bevy::render::render_graph::SlotInfo> {
        Vec::new()
    }

    fn update(&mut self, _world: &mut bevy::ecs::world::World) {}

    fn run<'w>(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w bevy::ecs::world::World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let windows = world.resource::<ExtractedWindows>();
        let primary_window = match windows.get(match &windows.primary {
            // If there is a primary window, use that
            Some(window_entity) => window_entity,
            // Otherwise there's no work to be done, so return
            None => return Ok(()),
        }) {
            // If the window was extracted properly, use that
            Some(primary_window) => primary_window,
            // Otherwise we don't have a window we can draw to, so just return.
            None => return Ok(()),
        };

        let shader_module = render_context
            .render_device()
            .create_and_validate_shader_module(ShaderModuleDescriptor {
                label: Some("WebGPU Triangle Shader"),
                source: ShaderSource::Wgsl(
                    include_str!("../assets/shader/webgpu_triangle.wesl").into(),
                ),
            });

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

        let render_pass_descriptor = RenderPassDescriptor {
            label: Some("WebGPU Triangle Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // Need to get the texture representing the window's surface. Probably stored in a resource in the render world.
                view: &primary_window
                    .swap_chain_texture_view
                    .as_ref()
                    .expect("There should be a swap chain texture"),
                depth_slice: None,
                resolve_target: None,
                // ops: default(),
                // Can't do this, wgpu_types seems to be inaccessible
                // Operations::<wgpu_types::Color> {
                //     load: todo!(),
                //     store: todo!(),
                // },
                ops: Operations {
                    load: LoadOp::Clear(Color::WHITE.to_linear().into()),
                    store: StoreOp::Store,
                }, // This above is how it's supposed to be done!
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        };

        let mut render_pass = render_context
            .command_encoder()
            .begin_render_pass(&render_pass_descriptor);
        render_pass.set_pipeline(&pipeline);
        render_pass.draw(0..3, 0..1);
        drop(render_pass);
        // let finished_command_queue = render_context.command_encoder().finish();
        // In the wgpu example they call queue.submit([finished_command_queue]) where queue is a stored wgpu::Queue.
        // Render context doesn't seem to have access to that though?
        // NOTE: The queue is probably submitted *after* the render graph has finished executing.
        //  See the `RenderGraphRunner` in bevy_render/renderer/graph_runner. Code is hard to follow though, look into this more.
        Ok(())
    }
}
