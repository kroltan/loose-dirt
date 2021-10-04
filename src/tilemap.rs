use bevy::{
    ecs::{component::Component, system::EntityCommands},
    prelude::*,
    reflect::TypeUuid,
    render::{
        pipeline::PipelineDescriptor,
        render_graph::{base::node::MAIN_PASS, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::ShaderStages,
        texture::{
            AddressMode, Extent3d, TextureDimension, TextureFormat,
        },
    },
};

use crate::GameStage;

pub struct TilemapPlugin<Tile> {
    scale: f32,
    width: isize,
    height: isize,
    template: Tile,
}

impl<Tile> TilemapPlugin<Tile> {
    pub fn new(width: usize, height: usize, scale: f32, template: Tile) -> Self {
        Self {
            width: width as isize,
            height: height as isize,
            scale,
            template,
        }
    }
}

impl<Tile: Component + Copy> Plugin for TilemapPlugin<Tile> {
    fn build(&self, app: &mut AppBuilder) {
        let template = self.template;

        let surface = {
            let mut texture = Texture::new_fill(
                Extent3d {
                    width: self.width as u32,
                    height: self.height as u32,
                    depth: 32,
                },
                TextureDimension::D2,
                &[0],
                TextureFormat::R8Uint,
            );

            texture.sampler.set_address_mode(AddressMode::ClampToEdge);

            app.world_mut()
                .get_resource_mut::<Assets<Texture>>()
                .unwrap()
                .add(texture)
        };

        app.insert_resource(Tilemap {
            scale: self.scale,
            width: self.width,
            height: self.height,
            content: vec![Entity::new(0); self.width as usize * self.height as usize]
                .into_boxed_slice(),
            initializer: Box::new(move |commands| {
                commands.insert(template);
            }),
            surface,
        });

        app.init_resource::<TilemapContext>();

        app.add_startup_system(init.system());

        app.add_system_to_stage(GameStage::Tally, neighbours::<Tile>.system());

        app.add_system_to_stage(GameStage::Tally, sync_surface.system());
    }
}

pub struct Tilemap {
    scale: f32,
    width: isize,
    height: isize,
    content: Box<[Entity]>,
    initializer: Box<dyn Fn(&mut EntityCommands) + Send + Sync>,
    surface: Handle<Texture>,
}

impl Tilemap {
    pub fn px_to_cell(&self, position: Vec2) -> (isize, isize) {
        let (x, y) = ((position - Vec2::ONE) / self.scale).into();
        (x.round() as isize, y.round() as isize)
    }

    pub fn iter(&self) -> impl Iterator<Item = (isize, isize)> {
        let width = self.width;
        let height = self.height;

        (0..height).flat_map(move |y| (0..width).map(move |x| (x, y)))
    }

    pub fn get(&self, x: isize, y: isize) -> Option<Entity> {
        if x < 0 || y < 0 || x >= self.width || y >= self.width {
            return None;
        }

        self.content.get(self.index(x, y)).cloned()
    }

    fn get_mut(&mut self, x: isize, y: isize) -> &mut Entity {
        &mut self.content[self.index(x, y)]
    }

    fn index(&self, x: isize, y: isize) -> usize {
        (y * self.width + x) as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TilePosition(pub isize, pub isize);

#[derive(Debug)]
pub struct LeftNeighbour<T: Component>(pub T);

#[derive(Debug)]
pub struct RightNeighbour<T: Component>(pub T);

#[derive(Debug)]
pub struct UpNeighbour<T: Component>(pub T);

#[derive(Debug)]
pub struct DownNeighbour<T: Component>(pub T);

#[derive(Debug)]
pub struct Material(pub u8);

#[derive(RenderResources, TypeUuid)]
#[uuid = "fe4aadbc-34d5-438f-8607-c92f5d856445"]
struct TilemapContext {
    #[render_resources(ignore)]
    pipeline: Handle<PipelineDescriptor>,
    time: f32,
    texel_size: Vec2,
}

impl FromWorld for TilemapContext {
    fn from_world(world: &mut World) -> Self {
        let server = world.get_resource::<AssetServer>().unwrap();
        let vertex = server.load("tilemap.vert");
        let fragment = server.load("tilemap.frag");
        let pipeline = world
            .get_resource_mut::<Assets<PipelineDescriptor>>()
            .unwrap()
            .add(PipelineDescriptor::default_config(ShaderStages {
                vertex,
                fragment: Some(fragment),
            }));

        let mut render_graph = world.get_resource_mut::<RenderGraph>().unwrap();
        let graph_node_name = "tilemap";

        render_graph.add_system_node(
            graph_node_name,
            RenderResourcesNode::<TilemapContext>::new(true),
        );

        render_graph
            .add_node_edge(graph_node_name, MAIN_PASS)
            .unwrap();

        let Tilemap { width, height, .. } = *world.get_resource().unwrap();

        Self {
            pipeline,
            time: 0.0,
            texel_size: Vec2::new(1.0 / width as f32, 1.0 / height as f32),
        }
    }
}

fn neighbours<Tile: Component + Copy>(
    mut commands: Commands,
    tilemap: Res<Tilemap>,
    tiles: Query<(&TilePosition, &Tile), Changed<Tile>>,
) {
    for (&TilePosition(x, y), target) in tiles.iter() {
        mark_neighbour(
            &mut commands,
            &tilemap,
            target,
            x,
            y - 1,
            UpNeighbour::<Tile>,
        );
        mark_neighbour(
            &mut commands,
            &tilemap,
            target,
            x,
            y + 1,
            DownNeighbour::<Tile>,
        );
        mark_neighbour(
            &mut commands,
            &tilemap,
            target,
            x + 1,
            y,
            LeftNeighbour::<Tile>,
        );
        mark_neighbour(
            &mut commands,
            &tilemap,
            target,
            x - 1,
            y,
            RightNeighbour::<Tile>,
        );
    }
}

fn mark_neighbour<T: Component + Copy, C: Component>(
    commands: &mut Commands,
    tilemap: &Tilemap,
    tile: &T,
    x: isize,
    y: isize,
    constructor: impl Fn(T) -> C,
) {
    if let Some(entity) = tilemap.get(x, y) {
        commands.entity(entity).insert(constructor(*tile));
    }
}

fn init(
    mut commands: Commands,
    mut tilemap: ResMut<Tilemap>,
    mut colors: ResMut<Assets<ColorMaterial>>,
    context: Res<TilemapContext>,
) {
    commands.spawn().insert_bundle(SpriteBundle {
        render_pipelines: RenderPipelines::from_handles([&context.pipeline]),
        material: colors.add(ColorMaterial::texture(tilemap.surface.clone())),
        sprite: Sprite::new(Vec2::new(tilemap.width as f32, tilemap.height as f32) * tilemap.scale),
        ..Default::default()
    });

    for (x, y) in tilemap.iter() {
        let mut builder = commands.spawn();

        builder.insert(TilePosition(x, y));
        builder.insert(Material(0));

        (tilemap.initializer)(&mut builder);

        *tilemap.get_mut(x, y) = builder.id();
    }
}

fn sync_surface(
    time: Res<Time>,
    tilemap: Res<Tilemap>,
    mut context: ResMut<TilemapContext>,
    mut textures: ResMut<Assets<Texture>>,
    pixels: Query<(&TilePosition, &Material), Changed<Material>>,
) {
    let surface = textures.get_mut(tilemap.surface.clone()).unwrap();

    let width = tilemap.width as usize;
    let height = tilemap.height as usize;

    for (&TilePosition(x, y), material) in pixels.iter() {
        let start = (height - y as usize) * width + x as usize;
        surface.data[start] = material.0;
    }

    context.time = time.seconds_since_startup() as f32;
}
