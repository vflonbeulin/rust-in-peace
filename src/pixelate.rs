use bevy::render::render_resource::BindGroupLayoutDescriptor;
use bevy::{
    core_pipeline::{
        core_3d::graph::{Core3d, Node3d},
        FullscreenShader,
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, FragmentState, MultisampleState, Operations,
            PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType,
        },
        renderer::{RenderContext, RenderDevice},
        view::ViewTarget,
        RenderApp,
    },
};

pub struct PixelatePlugin;

impl Plugin for PixelatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<PixelateSettings>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };

        render_app
            .add_render_graph_node::<ViewNodeRunner<PixelateNode>>(Core3d, PixelateLabel)
            .add_render_graph_edges(Core3d, (Node3d::Tonemapping, PixelateLabel, Node3d::EndMainPassPostProcessing));
    }

    fn finish(&self, app: &mut App) {
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.init_resource::<PixelatePipeline>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PixelateLabel;

#[derive(Component, Default, Clone, Copy, ExtractComponent, Reflect)]
pub struct PixelateSettings {
    pub block_size: f32,
}

#[derive(ShaderType)]
struct PixelateUniform {
    block_size: f32,
}

#[derive(Default)]
struct PixelateNode;

impl ViewNode for PixelateNode {
    type ViewQuery = (&'static ViewTarget, &'static PixelateSettings);

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, settings): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline = world.resource::<PixelatePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
            return Ok(());
        };

        let post_process = view_target.post_process_write();

        let uniform = PixelateUniform {
            block_size: settings.block_size,
        };
        let uniform_buffer = render_context.render_device().create_buffer_with_data(
            &bevy::render::render_resource::BufferInitDescriptor {
                label: Some("pixelate_uniform"),
                contents: &uniform.block_size.to_ne_bytes().iter()
                    .chain([0u8; 12].iter()) // padding for alignment
                    .copied()
                    .collect::<Vec<_>>(),
                usage: bevy::render::render_resource::BufferUsages::UNIFORM,
            },
        );

        let bind_group = render_context.render_device().create_bind_group(
            "pixelate_bind_group",
            &pipeline.layout,
            &BindGroupEntries::sequential((
                post_process.source,
                &pipeline.sampler,
                uniform_buffer.as_entire_binding(),
            )),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("pixelate_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
                depth_slice: None, 
            })],
            ..default()
        });

        render_pass.set_render_pipeline(render_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

#[derive(Resource)]
struct PixelatePipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for PixelatePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout_entries = BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<PixelateUniform>(false),
            ),
        );

        let layout = render_device.create_bind_group_layout(
            "pixelate_bind_group_layout",
            &layout_entries,
        );

        let layout_descriptor = BindGroupLayoutDescriptor {
            label: "pixelate_bind_group_layout".into(),
            entries: layout_entries.to_vec(),
        };

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.load_asset("shaders/pixelate.wgsl");
        
        let vertex = world.resource::<FullscreenShader>().to_vertex_state();

        let pipeline_id = world.resource_mut::<PipelineCache>().queue_render_pipeline(
            RenderPipelineDescriptor {
                label: Some("pixelate_pipeline".into()),
                layout: vec![layout_descriptor],
                vertex,
                fragment: Some(FragmentState {
                    shader,
                    shader_defs: vec![],
                    entry_point: Some("fragment".into()),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::Rgba8UnormSrgb,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
                zero_initialize_workgroup_memory: false,
            },
        );

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
