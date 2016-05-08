extern crate euclid;
#[macro_use]
extern crate glium;

extern crate vitral;

use std::mem;
use std::rc::Rc;

use glium::{Surface, GlObject};
use glium::glutin;
use glium::index::PrimitiveType;
use euclid::{Rect, Point2D, Size2D};

use vitral::Context;

type GliumTexture = glium::texture::CompressedSrgbTexture2d;

struct Texture(GliumTexture);

impl PartialEq<Texture> for Texture {
    fn eq(&self, other: &Texture) -> bool {
        self.0.get_id() == other.0.get_id()
    }
}

// XXX: An exact copy of Vitral vertex struct, just so that I can derive a
// Glium vertex implementatino for it.
#[derive(Copy, Clone)]
pub struct GliumVertex {
    pub pos: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2],
}
implement_vertex!(GliumVertex, pos, color, tex);

fn main() {
    use glium::DisplayBuild;

    // building the display, ie. the main object
    let display = glutin::WindowBuilder::new()
                      .build_glium()
                      .unwrap();

    let image = glium::texture::RawImage2d::from_raw_rgba(vec![0xffffffffu32], (1, 1));
    let opengl_texture = Rc::new(Texture(glium::texture::CompressedSrgbTexture2d::new(&display,
                                                                                      image)
                                             .unwrap()));

    // compiling shaders and linking them together
    let program = program!(&display,
        150 => {
            vertex: "
                #version 150 core

                uniform mat4 matrix;

                in vec2 pos;
                in vec4 color;
                in vec2 tex;

                out vec4 vColor;
                out vec2 vTexcoord;

                void main() {
                    gl_Position = vec4(pos, 0.0, 1.0) * matrix;
                    vColor = color;
                    vTexcoord = tex;
                }
            ",

            fragment: "
                #version 150 core
                uniform sampler2D tex;
                in vec4 vColor;
                in vec2 vTexcoord;
                out vec4 f_color;

                void main() {
                    f_color = vColor * texture(tex, vTexcoord);
                }
            "
        },
    )
                      .unwrap();

    let mut context = Context::new(opengl_texture.clone());

    // the main loop
    loop {
        context.begin_frame();
        let area = Rect::new(Point2D::new(10.0, 10.0), Size2D::new(16.0, 16.0));
        context.fill_rect(area, [0.0, 0.0, 0.0, 1.0]);
        context.fill_rect(area.inflate(-1.0, -1.0), [1.0, 0.0, 0.0, 1.0]);

        // drawing a frame

        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);
        let (w, h) = target.get_dimensions();

        for batch in context.end_frame() {
            // building the uniforms
            let uniforms = uniform! {
                matrix: [
                    [2.0 / w as f32, 0.0, 0.0, -1.0],
                    [0.0, -2.0 / h as f32, 0.0, 1.0],
                    [0.0, 0.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0, 1.0f32]
                ],
                tex: &(*batch.texture).0,
            };

            let vertex_buffer = {
                glium::VertexBuffer::new(&display,
                                         // XXX: Have to do the unsafe switcheroo here to get a
                                         // vertex type with Glium traits derived for it.
                                         &unsafe {
                                             mem::transmute::<Vec<vitral::Vertex>,
                                                              Vec<GliumVertex>>(batch.vertices)
                                         })
                    .unwrap()
            };

            // building the index buffer
            let index_buffer = glium::IndexBuffer::new(&display,
                                                       PrimitiveType::TrianglesList,
                                                       &batch.triangle_indices)
                                   .unwrap();

            let params = glium::draw_parameters::DrawParameters {
                scissor: batch.clip.map(|clip| {
                    glium::Rect {
                        left: clip.origin.x as u32,
                        bottom: h - (clip.origin.y + clip.size.height) as u32,
                        width: clip.size.width as u32,
                        height: clip.size.height as u32,
                    }
                }),
                ..Default::default()
            };

            target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &params).unwrap();
        }

        target.finish().unwrap();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => (),
            }
        }
    }
}