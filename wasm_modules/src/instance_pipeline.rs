/*
instance_pipeline.rs

custom render pipeline that sets up the shaders
important for instancing our data
*/

use bevy::{
    prelude::*,
    core_pipeline::core_3d::Transparent3d,
    ecs::{
        system::{lifetimeless::*, SystemParamItem},
    },
    pbr::{
        MeshPipeline, MeshPipelineKey, RenderMeshInstances, 
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    render::{
        extract_component::ExtractComponentPlugin,
        extract_resource::ExtractResourcePlugin,
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
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
        view::{
            ExtractedView, 
        },
        Render, RenderApp, RenderSet,
    },
};
use std::mem::size_of;

// stuff we buffer into gpu
use crate::earth::{
    AllMapData,
    InstanceData,
    InstanceMaterialData,
};
use crate::sun::SunPosition;
use crate::mapupdate::CurrentMap;

// setup -> ECS entities with CPU side data
// extract ->entities from main world and copy to render world
// prepare -> CPU data and turn into GPU buffers
// queue -> what should be drawn and how using pipeline with draw function 
// draw -> render using the shader

// plugin ---------------------------------------------------------------------

// in assets subdirectory
const SHADER_ASSET_PATH: &str = "shaders/instancing.wgsl";
//const SHADER_ASSET_PATH: &str = "shaders/test.wgsl";

// custom render pipeline plugin
pub struct CustomMaterialPlugin;

//
impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        // all the extractions we need to make from game world to render world
        app.add_plugins((
            ExtractComponentPlugin::<InstanceMaterialData>::default(),
            ExtractResourcePlugin::<AllMapData>::default(),
            ExtractResourcePlugin::<CurrentMap>::default(),
            ExtractResourcePlugin::<SunPosition>::default(),
        ));
        // describing all the parts of the pipeline
        // renderapp with a custom pipeline resource
        // render systems are exectuted 
        app.sub_app_mut(RenderApp)
            .add_render_command::<Transparent3d, DrawCustom>()
            .init_resource::<SpecializedMeshPipelines<CustomPipeline>>()
            .add_systems(
                Render,
                (
                    prepare_instance_buffers
                        .in_set(RenderSet::PrepareResources),
                    
                    update_map_from_main_world
                        .in_set(RenderSet::PrepareResources)
                        .after(prepare_instance_buffers),

                    update_sun_from_main_world
                        .in_set(RenderSet::PrepareResources)
                        .after(prepare_instance_buffers),
                    
                    queue_custom.in_set(RenderSet::QueueMeshes),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<CustomPipeline>();
    }
}

// GPU resources

// what gets copied to GPU
#[derive(Component)]
struct InstanceBuffer {
    buffer: Buffer,
    length: usize,
}

#[derive(Resource)]
struct ElevationStorageBuffer {
    buffer: Buffer,
}

#[derive(Resource)]
pub struct MapSelectionUniformBuffer {
    pub buffer: Buffer,
    pub current_map: u32,
}

#[derive(Resource)]
pub struct SunPositionUniformBuffer {
    pub buffer: Buffer,
    pub position: Vec3,
}

#[derive(Resource)]
struct ElevationBindGroup {
    bind_group: BindGroup,
}


// PREPARE --------------------------------------------------------------------

// turn our vec stuff into raw bytes
// does this need to be run as a render system or can it run once?
fn prepare_instance_buffers(
    mut commands: Commands,
    query: Query<(Entity, &InstanceMaterialData)>,
    render_device: Res<RenderDevice>,
    all_maps: Option<Res<AllMapData>>,
    existing_elevation: Option<Res<ElevationStorageBuffer>>,
    existing_map_selection: Option<Res<MapSelectionUniformBuffer>>,
    existing_sun_position: Option<Res<SunPositionUniformBuffer>>,
    mut count: Local<Option<u64>>,
) {
    let count = count.get_or_insert(0);

    let instance_count = query.iter().count();
    //println!("prepare_instance_buffers: Found {} entities with InstanceMaterialData", instance_count);

    //if entity InstanceBuffer exists, do not run
    // or do you have to run it each time?
    // same with the flattening of the storage buffer, thats not a render thing
    // thats more for preparation before render
    


    // instance buffer creation
    for (entity, instance_data) in &query {
        //*count += 1;
        //println!("Creating instance buffer for entity {:?} with {} instances | run {}|", 
        //    entity, instance_data.len(), count);

        let instance_buffer = 
        render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("instance data buffer"),
            contents: bytemuck::cast_slice(instance_data.as_slice()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });
        commands.entity(entity).insert(InstanceBuffer {
            buffer: instance_buffer,
            length: instance_data.len(),
        });
    }

    // flatten storage buffer
    if let Some(all_maps) = all_maps {
        if existing_elevation.is_none() {
            println!("Creating elevation storage buffer with {} maps", all_maps.maps.len());
            let all_elevations: Vec<i32> = all_maps.maps
                .iter()
                .flat_map(|map| &map.buffer)
                .copied()
                .collect();
            println!("Total elevation points: {}", all_elevations.len());
            let elevation_buffer =
            render_device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("elevation storage buffer"),
                contents: bytemuck::cast_slice(&all_elevations),
                usage: BufferUsages::STORAGE,
            });
            commands.insert_resource(ElevationStorageBuffer {
                buffer: elevation_buffer,
            });
        }
    } else {
        println!("WARNING: AllMapData resource not found!");
    }

    // create map selection uniform buffer
    if existing_map_selection.is_none() {
        println!("Creating map selection uniform buffer");

        // defaults at 0
        // map_id, points_per_map, padding
        let map_data: [u32; 4] = [0, 65341, 0, 0];

        let map_buffer =
        render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("map selection uniform buffer"),
            contents: bytemuck::cast_slice(&map_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        commands.insert_resource(MapSelectionUniformBuffer { 
            buffer: map_buffer,
            current_map: 0,
        });
    }

    // create sun position uniform buffer
    if existing_sun_position.is_none() {
        println!("Creating sun position uniform buffer");

        // defaults at 149k
        let sun_data: [f32; 4] = [149_000.0, 0.0, 0.0, 1.0];

        let sun_buffer =
        render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("sun position uniform buffer"),
            contents: bytemuck::cast_slice(&sun_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        commands.insert_resource(SunPositionUniformBuffer { 
            buffer: sun_buffer,
            position: Vec3::new(149_000.0, 0.0, 0.0),
        });
    }
}

// PIPELINE -------------------------------------------------------------------

// defining the custom pipeline, uses standard bevy PBR pipeline as base

// what are the other parts of a pipeline you can customize?
#[derive(Resource)]
struct CustomPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    elevation_bind_group_layout: BindGroupLayout,
}

//the pipeline has functions that need to be implemented

// load the shader and get base pipeline
impl FromWorld for CustomPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let mesh_pipeline = world.resource::<MeshPipeline>();

        // create bind group layout for elevation data
        // the point of these layouts is that the gpu knows what data
        // its dealing with.
        // we declare what binding slot, what shader stage can use it, 
        // what type of bind is it?, storage? (not allowed in webgl2)
        // uniform also exists, whats the diff?
        // we need a texture and sampler for webgl2 (not a bad idea)
        let elevation_bind_group_layout = 
        render_device.create_bind_group_layout(
            Some("elevation_bind_group_layout"),
            &[
                // binding 0: elevation storage buffer
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {read_only: true},
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 1: map selection uniform buffer
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // binding 2: sun position uniform buffer
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT, //VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        );
        // return the pipeline
        CustomPipeline {
            shader: world.load_asset(SHADER_ASSET_PATH),
            mesh_pipeline: mesh_pipeline.clone(),
            elevation_bind_group_layout,
        }
    }
}

// our specialized mesh pipeline logic, overriding standard pbr
// this deals with geometry
impl SpecializedMeshPipeline for CustomPipeline {
    type Key = MeshPipelineKey;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        // get descriptor from standard, then we modify it
        let mut descriptor = self.mesh_pipeline.specialize(key, layout)?;

        // override the shaders using clones from struct
        // why two versions of the same thing?
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();

        // add our custom bind group layout to the pipeline layout
        // the standard mesh pipeline uses bind groups 0 and 1, so we add ours at position 2
        descriptor.layout.push(self.elevation_bind_group_layout.clone());

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
                // by Position, Normal and UV attributes of the base mesh
                // vec3, starts at byte 0
                // @location(3) in WGSL
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 3,                 
                },
                // scale
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 4,
                },
                //rotation
                //using manual offsets here at byte 16
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
                // can remove?
                // color
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 6,
                },
                // elevation index
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: 64,
                    shader_location: 7,
                }
                // is there anything else we want to precompute once?
                // normals? or are they the same? no theres rotation
                // elevation movement won't affect normals?
            ],
        });
        Ok(descriptor)
    }
}


// is QUEUE and DRAW part of custom RENDER?

// QUEUE ----------------------------------------------------------------------

// function that queues up all of our custom stuff in the render pipeline
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
    render_device: Res<RenderDevice>,
    elevation_storage: Option<Res<ElevationStorageBuffer>>,
    map_selection: Option<Res<MapSelectionUniformBuffer>>,
    sun_position: Option<Res<SunPositionUniformBuffer>>,
    existing_bind_group: Option<Res<ElevationBindGroup>>,
    mut commands: Commands,
) {
    // add our bind layouts to bind groups
    // how to make this scale?
    if let (Some(elevation_storage), Some(map_selection), Some(sun_position)) = 
    (elevation_storage, map_selection, sun_position) {
        if existing_bind_group.is_none() {
            let bind_group = render_device.create_bind_group(
                // maybe not the best name?
                Some("elevation_bind_group"),
                // we have elevations here, the layout
                &custom_pipeline.elevation_bind_group_layout,
                // then also the here plus the map selection?
                &[
                    BindGroupEntry {
                        binding: 0,
                        resource: elevation_storage.buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: map_selection.buffer.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: sun_position.buffer.as_entire_binding(),
                    },
                ]
            );

            commands.insert_resource(ElevationBindGroup { bind_group });
        }
    }
    
    // same as default draw?
    let draw_custom = transparent_3d_draw_functions.read().id::<DrawCustom>();

    // for each camera/view, we have one camera in our scene
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

        // for each entity without instancing data
        // not exactly sure whats going on here
        for (entity, main_entity) in &material_meshes {
            // so we have all of our mesh instances?
            let Some(mesh_instance) = render_mesh_instances
                                        .render_mesh_queue_data(*main_entity)
            else {
                continue;
            };

            let Some(mesh) = meshes.get(mesh_instance.mesh_asset_id) else {
                continue;
            };
            // what is a key?
            // what is | in this situation?
            let key =
                view_key | MeshPipelineKey::from_primitive_topology(
                                                mesh.primitive_topology());
            
            // once all the setup is done
            // specialize and compile pipeline for current view settings
            let pipeline = pipelines
                .specialize(
                    &pipeline_cache, &custom_pipeline, 
                    key, &mesh.layout
                )
                .unwrap();

            // add new drawing command to render phase
            // using our pipeline and draw function
            // transparent phase is when stuff isn't visible to camera yet?
            // so entity (cam/view), is bound to a pipeline with a draw func
            // distance? this other stuff?
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


    // how to pass this to draw command?
    // does it need to be passed? this is not a command but
    // an order in which things need to occur?
}

// DRAW -----------------------------------------------------------------------

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
    SetElevationBindGroup<2>,
    DrawMeshInstanced,
);

struct SetElevationBindGroup<const I: usize>;

// this whole signature of impl, fn, stuff i need to learn
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetElevationBindGroup<I> {
    // custom types per impl?
    type Param = Option<SRes<ElevationBindGroup>>;
    type ViewQuery = ();
    type ItemQuery = ();

    fn render<'w>(
        item: &P,
        _view: (),
        _item_query: Option<()>,
        elevation_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        /*
        let bind_group = elevation_bind_group.into_inner();
        if let Some(bind_group) = bind_group {
            pass.set_bind_group(I, &bind_group.bind_group, &[]);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Skip
        }
        */
        // cause single Sres, have to surround with Option
        // but this causes error in the into_inner
        // so we Some the wrapped option first, then we can access inside Sres?
        if let Some(elevation_bind_group) = elevation_bind_group {
            let elevation_bind_group = elevation_bind_group.into_inner();
            pass.set_bind_group(I, &elevation_bind_group.bind_group, &[]);
            RenderCommandResult::Success
        } else {
            // If the resource isn't ready, skip this draw call.
            RenderCommandResult::Skip
        }
    }
}


// this looks like each PhaseItem is an instance to render?

struct DrawMeshInstanced;

impl<P: PhaseItem> RenderCommand<P> for DrawMeshInstanced {
    type Param = (
        SRes<RenderAssets<RenderMesh>>,
        SRes<RenderMeshInstances>,
        SRes<MeshAllocator>,
    );
    type ViewQuery = ();
    type ItemQuery = Read<InstanceBuffer>;

    // what does inline do?
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

// MAP SWITCH
//
fn update_map_from_main_world(
    // We no longer need to access the whole MainWorld.
    // Instead, we get the resources that have been extracted for us.
    render_queue: Res<RenderQueue>,
    // This is the CurrentMap that was copied from the Main World.
    // We use Option<> in case it doesn't exist on the very first frame.
    current_map: Option<Res<CurrentMap>>,
    mut map_selection: Option<ResMut<MapSelectionUniformBuffer>>,
) {
    // This first print statement can now be removed or kept for debugging.
    // println!("checking map_selection");

    // Check that both the extracted resource and our render buffer exist.
    if let (Some(current_map), Some(mut map_selection)) = 
    (current_map, map_selection) {
        
        // This is the core logic. Has the index in the Main World changed
        // from what the Render World currently knows?
        if map_selection.current_map != current_map.index as u32 {
            println!("Map index changed! Updating GPU buffer from {} to {}.", map_selection.current_map, current_map.index);

            let new_map_id = current_map.index as u32;
            
            // Update our state in the Render World
            map_selection.current_map = new_map_id;
            
            // Write the new data to the actual GPU buffer.
            let data: [u32; 4] = [new_map_id, 65341, 0, 0];
            render_queue.write_buffer(
                &map_selection.buffer,
                0,
                bytemuck::cast_slice(&data),
            );
        }
    }
}

fn update_sun_from_main_world(
    render_queue: Res<RenderQueue>,
    sun_position: Option<Res<SunPosition>>,
    mut sun_position_buffer: Option<ResMut<SunPositionUniformBuffer>>,
) {
    if let (Some(sun_position), Some(mut sun_position_buffer)) =
    (sun_position, sun_position_buffer) {
            let n = sun_position.pos;
            sun_position_buffer.position = n;

            let data: [f32; 4] = [n.x, n.y, n.z, 1.0];
            render_queue.write_buffer(
                &sun_position_buffer.buffer,
                0,
                bytemuck::cast_slice(&data),
            );
    }
}

