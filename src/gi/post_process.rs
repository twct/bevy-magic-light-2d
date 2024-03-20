use bevy::{
    core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state,
    prelude::*,
    render::{
        extract_component::{ComponentUniforms, ExtractComponent},
        render_asset::RenderAssets,
        render_graph::{RenderLabel, ViewNode},
        render_resource::{
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment,
            RenderPassDescriptor, RenderPipelineDescriptor, Sampler, SamplerBindingType,
            SamplerDescriptor, ShaderStages, ShaderType, TextureFormat, TextureSampleType,
            TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ViewTarget,
    },
};

use super::{pipeline::GiTargetsWrapper, SHADER_POST_PROCESS};

#[derive(Default, Debug, Clone, Hash, PartialEq, Eq, RenderLabel)]
pub struct LightPostProcessNode;

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
pub struct LightPostProcessSettings {
    pub time: f32,
}

#[derive(Resource)]
pub struct LightPostProcessPipeline {
    pub layout: BindGroupLayout,
    pub sampler: Sampler,
    pub pipeline_id: CachedRenderPipelineId,
}

impl ViewNode for LightPostProcessNode {
    type ViewQuery = &'static ViewTarget;

    fn run<'w>(
        &self,
        graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        view_query: bevy::ecs::query::QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        let post_process_pipeline = world.resource::<LightPostProcessPipeline>();

        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(post_process_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<LightPostProcessSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let gi_targets_wrapper = world.resource::<GiTargetsWrapper>();

        let image_handle = gi_targets_wrapper
            .targets
            .as_ref()
            .unwrap()
            .ss_filter_target
            // .ss_bounce_target
            // .ss_pose_target
            // .ss_probe_target
            // .ss_blend_target
            .clone(); // Clone the handle to use it for lookup

        let texture_views = world.resource::<RenderAssets<Image>>();
        let irradiance_texture_view = if let Some(gpu_image) = texture_views.get(&image_handle) {
            &gpu_image.texture_view
        } else {
            // Handle the case where the texture view is not available yet
            return Ok(());
        };

        let post_process = view_query.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "light_post_process_bind_group",
            &post_process_pipeline.layout,
            // It's important for this to match the BindGroupLayout defined in the PostProcessPipeline
            &BindGroupEntries::sequential((
                // Make sure to use the source view
                post_process.source,
                // Use the sampler created for the pipeline
                &post_process_pipeline.sampler,
                // Set the settings binding
                settings_binding.clone(),
                BindingResource::TextureView(irradiance_texture_view),
                BindingResource::Sampler(&post_process_pipeline.sampler),
            )),
        );

        // Begin the render pass
        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("light_post_process_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                // We need to specify the post process destination view here
                // to make sure we write to the appropriate texture.
                view: post_process.destination,
                resolve_target: None,
                ops: Operations::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // This is mostly just wgpu boilerplate for drawing a fullscreen triangle,
        // using the pipeline/bind_group created above
        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}

impl FromWorld for LightPostProcessPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "light_post_process_bind_group_layout",
            // The screen texture
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // The sampler that will be used to sample the screen texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // The settings uniform that will control the effect
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: bevy::render::render_resource::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(LightPostProcessSettings::min_size()),
                    },
                    count: None,
                },
                // Irradiance image
                BindGroupLayoutEntry {
                    binding: 3, // Texture binding
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler for the irradiance image
                BindGroupLayoutEntry {
                    binding: 4, // Next available binding index for the sampler
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        );

        // We can create the sampler here since it won't change at runtime and doesn't depend on the view
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let pipeline_id = world
            .resource_mut::<PipelineCache>()
            // This will add the pipeline to the cache and queue it's creation
            .queue_render_pipeline(RenderPipelineDescriptor {
                label: Some("light_post_process_pipeline".into()),
                layout: vec![layout.clone()],
                // This will setup a fullscreen triangle for the vertex state
                vertex: fullscreen_shader_vertex_state(),
                fragment: Some(FragmentState {
                    shader: SHADER_POST_PROCESS.clone(),
                    shader_defs: vec![],
                    // Make sure this matches the entry point of your shader.
                    // It can be anything as long as it matches here and in the shader.
                    entry_point: "fragment".into(),
                    targets: vec![Some(ColorTargetState {
                        format: TextureFormat::bevy_default(),
                        // format: TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                // All of the following properties are not important for this effect so just use the default values.
                // This struct doesn't have the Default trait implemented because not all field can have a default value.
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                push_constant_ranges: vec![],
            });

        Self {
            layout,
            sampler,
            pipeline_id,
        }
    }
}
