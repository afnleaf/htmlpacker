use bevy::{
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        query::QueryItem,
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, 
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        mesh::{
            allocator::MeshAllocator, MeshVertexBufferLayoutRef, 
            RenderMesh, RenderMeshBufferInfo,
        },
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, PhaseItemExtraIndex, 
            RenderCommand, RenderCommandResult, SetItemPipeline, 
            TrackedRenderPass, ViewSortedRenderPhases,
        },
        render_resource::*,
        renderer::RenderDevice,
        sync_world::MainEntity,
        view::{
            ExtractedView, 
            NoFrustumCulling, 
            //NoIndirectDrawing
        },
        Render, RenderApp, RenderSet,
    },
};
use bytemuck::{Pod, Zeroable};

// setup -> ECS entities with CPU side data
// extract ->entities from main world and copy to render world
// prepare -> CPU data and turn into GPU buffers
// queue -> what should be drawn and how using pipeline with draw function 
// draw -> render using the shader

// in assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/instancing.wgsl";


// the buffer that gets sent to the shader
#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct InstanceData {
    position: Vec3,
    scale: f32,
    rotation: Quat,
    color: [f32; 4],
}

// all of the instance properties together in a vec as a component
// deref trait when used for a struct wrap around lets the compiler know
// that if a method or field is used on this struct but doesn't exist
// look at the type that it wraps
#[derive(Component, Deref)]
struct InstanceMaterialData(Vec<InstanceData>);

// getting out each material component from the vec of instances
// bridge between main world (game logic) and render world (render logic) 
impl ExtractComponent for InstanceMaterialData {
    type QueryData = &'static InstanceMaterialData;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(
        item: QueryItem<'_, 
        Self::QueryData>
    ) -> Option<Self> {
        Some(InstanceMaterialData(item.0.clone()))
    }
}

// custom render pipeline plugin
pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            ExtractComponentPlugin::<InstanceMaterialData>::default()
        );
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    queue_custom.in_set(RenderSet::QueueMeshes),
                    prepare_instance_buffers.in_set(RenderSet::PrepareResources),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}


// our basic setup (bad)
pub fn setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // configure sphere parameters
    let height = 181;
    let width = 361;
    //let n = height * width;
    
    // sphere radius (6378km = earth radius)
    let r = 6.378_f64;
    
    // Generate sphere points and convert to InstanceData
    let instance_data: Vec<InstanceData> = (0..height)
        .flat_map(|i| {
            // Map i from [0, height-1] to [-90, 90] degrees (latitude)
            let lat_deg = -90.0 + (i as f64 * 180.0 / (height as f64 - 1.0));
            let lat_rad = lat_deg * std::f64::consts::PI / 180.0;
            
            (0..width).map(move |j| {
                // Map j from [0, width-1] to [-180, 180] degrees (longitude)
                let lon_deg = -180.0 + (j as f64 * 360.0 / (width as f64 - 1.0));
                let lon_rad = lon_deg * std::f64::consts::PI / 180.0;
                
                // Cartesian conversion
                let x = (r * lat_rad.cos() * lon_rad.cos()) as f32;
                let y = (r * lat_rad.cos() * lon_rad.sin()) as f32;
                let z = (r * lat_rad.sin()) as f32;

                let position = Vec3::new(x, y, z);
                let normal = position.normalize();
                // Calculate rotation to orient the cube outward from origin
                // This rotates from default Y-up to point along the normal
                let rotation = if normal.y.abs() > 0.999 {
                    // Special case for poles to avoid numerical instability
                    if normal.y > 0.0 {
                        Quat::IDENTITY
                    } else {
                        Quat::from_rotation_x(std::f32::consts::PI)
                    }
                } else {
                    Quat::from_rotation_arc(Vec3::Y, normal)
                };
                
                // Create instance data for this point
                InstanceData {
                    position,
                    scale: 0.01, // Adjust scale as needed for your cubes
                    rotation,
                    color: LinearRgba::from(Color::hsla(
                        (lon_deg + 180.0) as f32, // Hue based on longitude
                        0.7,                        // Saturation
                        0.5,                        // Lightness
                        1.0                         // Alpha
                    )).to_f32_array(),
                }
            })
        })
        .collect();

    println!("Creating {} instances", instance_data.len());
    println!("InstanceData size: {} bytes", std::mem::size_of::<InstanceData>());
    
    // Spawn the instanced mesh
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.2, 0.4, 0.2))),
        InstanceMaterialData(instance_data),
        NoFrustumCulling,
    ));
}

// what gets copied to GPU
#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

// turn our vec stuff into raw bytes
fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
) {
    for (entity, instance_data) in &query {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer,
            length: instance_data.len(),
        });
    }
}

// custom draw call
fn queue_custom(
    transparent_3d_draw_functions: Res<DrawFunctions<Transparent3d>>,
    custom_pipeline: Res<CustomPipeline>,
    mut pipelines: ResMut<SpecializedMeshPipelines<CustomPipeline>>,
    pipeline_cache: Res<PipelineCache>,
    meshes: Res<RenderAssets<RenderMesh>>,
    render_mesh_instances: Res<RenderMeshInstances>,
    material_meshes: Query<(Entity, &MainEntity), With<InstanceMaterialData>>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent3d>>,
    views: Query<(&ExtractedView, &Msaa)>,
) {
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();
    
    // for each camera/view
    for (view, msaa) in &views {
        // get list of things to draw 
        let Some(transparent_phase) = transparent_render_phases.get_mut(
                                                    &view.retained_view_entity)
        else {
            continue;
        };

        let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);
        let rangefinder = view.rangefinder3d();

        // for each entity witho out instancing data
        for (entity, main_entity) in &material_meshes {
            let Some(mesh_instance) = render_mesh_instances
                                        .render_mesh_queue_data(*main_entity)
            else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(
                                                mesh.primitive_topology());

            // specialize and compile pipeline for current view settings
            let pipeline = pipelines
                .specialize(
                    &pipeline_cache, &custom_pipeline, 
                    key, &mesh.layout
                )
                .unwrap();

            // add new drawing command to render phase
            // using our pipeline and draw function
            transparent_phase.add(Transparent3d {
                entity: (entity, *main_entity),
                pipeline,
                draw_function: draw_custom, 
                distance: rangefinder.distance_translation(
                            &mesh_instance.translation),
                batch_range: 0..1,
                extra_index: PhaseItemExtraIndex::None,
                indexed: true,
            });
        }
    }
}


// defining the custom pipeline
// uses standard bevy PBR pipeline as base
#[derive(Resource)]
struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
}

// load the shader and get base pipeline
impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let mesh_pipeline = world.resource::<MeshPipeline>();

        CustomPipeline {
            shader: world.load_asset(SHADER_ASSET_PATH),
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

// our specialized pipeline logic, overriding standard pbr
impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        // get descriptor from standard
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;
        // override the shaders
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        
        // ALERT
        // extremely important for instancing!!!
        // this adds a new vertex buffer layout with our custom instance data
        descriptor.vertex.buffers.push(VertexBufferLayout {
            // how many bytes to step forward for each instance
            array_stride: size_of::<InstanceData>() as u64,
            // advance per instance, not per vertex
            step_mode: VertexStepMode::Instance,
            // our attrivute and memory layout
            attributes: vec![
                // position
                // shader locations 0-2 are taken up 
                // by Position, Normal and UV attributes
                // vec4, starts at byte 0
                // @location(3) in WGSL
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,                 
                },
                //rotation
                //using manual offsets here at byte 16
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 4
                },
                // color
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
            ],
        });
        Ok(descriptor)
    }
}


// custom draw command
// these commands get run in order
// binds the pipeline we in specialized in queue_custom
// binds camera/view data
// binds mesh transform data
// our custom command to do actual rendering
type DrawCustom = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetMeshBindGroup<1>,
    DrawMeshInstanced,
);

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    #[inline]
    fn render<'w>(
        item: &P,
        _view: (),
        instance_buffer: Option<&'w InstanceBuffer>,
        (meshes, 
        render_mesh_instances, 
        mesh_allocator): SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        // get the mesh and our instance buffer component
        // A borrow check workaround.
        // kind of wonky looking
        let mesh_allocator = mesh_allocator.into_inner();

        let Some(mesh_instance) = render_mesh_instances
                                    .render_mesh_queue_data(item.main_entity())
        else {
            return RenderCommandResult::Skip;
        };
        let Some(gpu_mesh) = meshes.into_inner()
                                .get(mesh_instance.mesh_asset_id) 
        else {
            return RenderCommandResult::Skip;
        };

        let Some(instance_buffer) = instance_buffer else {
            return RenderCommandResult::Skip;
        };

        let Some(vertex_buffer_slice) =
            mesh_allocator.mesh_vertex_slice(&mesh_instance.mesh_asset_id)
        else {
            return RenderCommandResult::Skip;
        };

        // GPU slots are not slots on your motherboard
        // they are buffers you plug in with different properties
        // prism vertex buffer is the geometry (vertices, normals, uvs)
        // instance data is the positions, rotations, colors
        // so it follows that we have more than 2 slots?
        // fn specialize to desribe what each slot is for and expected data
        // DrawMeshInstanced (here) activates the slot and renders
        // this is highly important because you are writing instructions
        // for how CPU and GPU operate together, real physical connections 
        // need to be made for the bits to travel
        // the shader is the program you end up running on the GPU in parallel
        
        // bind prism vertex buffer to GPU slot 0
        pass.set_vertex_buffer(0, vertex_buffer_slice.buffer.slice(..));
        // bind instance data buffer to GPU slot 1
        pass.set_vertex_buffer(1, instance_buffer.buffer.slice(..));

        match &gpu_mesh.buffer_info {
            RenderMeshBufferInfo::Indexed {
                index_format,
                count,
            } => {
                let Some(index_buffer_slice) =
                    mesh_allocator.mesh_index_slice(&mesh_instance.mesh_asset_id)
                else {
                    return RenderCommandResult::Skip;
                };

                pass.set_index_buffer(
                    index_buffer_slice.buffer.slice(..), 
                    0, *index_format
                );

                // the final instanced draw call
                // indices to draw,
                // base vertex,
                pass.draw_indexed(
                    index_buffer_slice.range.start..(
                        index_buffer_slice.range.start + count
                    ),
                    vertex_buffer_slice.range.start as i32,
                    // draw instances from 0 up to total
                    0..instance_buffer.length as u32,
                );
            }
            RenderMeshBufferInfo::NonIndexed => {
                pass.draw(
                    vertex_buffer_slice.range, 
                    0..instance_buffer.length as u32
                );
            }
        }
        RenderCommandResult::Success
    }
}
