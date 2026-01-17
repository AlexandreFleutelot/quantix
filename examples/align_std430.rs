use quantix::helpers;

use wgpu;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct InputData {
    position : [f32; 3],
    speed : f32,
}

async fn test_std430() {
    let ctx = helpers::GpuContext::new().await;

    let input_data:InputData = InputData {
        position : [0.1, 0.2, 0.3],
        speed : 42.0,
    };

    //Definition du buffers avec le contenu de la struct directement
    let input_buffer = ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Input Buffer quantix"),
        contents: bytemuck::cast_slice(&[input_data]),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    });

    //Read from the buffer
    let data: Vec<InputData> = ctx.read_buffer::<InputData>(&input_buffer, std::mem::size_of::<InputData>() as u64).await;
    println!("read data: {:?}", data); // OK

    //Shader & Pipeline
    let shader_src = "
        struct Data {
            position: vec3<f32>,
            speed: f32,
        };

        @group(0) @binding(0) var<storage, read_write> data: Data;

        @compute @workgroup_size(1)
        fn main() {
            data.speed = data.position.x;
        }
    ";

    let shader = ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(shader_src.into()),
    });

    let pipeline = ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: input_buffer.as_entire_binding(),
        }],
    });

    // Exécution
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    {
        let mut cpass = encoder.begin_compute_pass(&Default::default());
        cpass.set_pipeline(&pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(1, 1, 1);
    }
    ctx.queue.submit(Some(encoder.finish()));

    //Read from the buffer
    let data: Vec<InputData> = ctx.read_buffer::<InputData>(&input_buffer, std::mem::size_of::<InputData>() as u64).await;
    println!("read data (after shader): {:?}", data);
    println!("Vitesse attendue (position.x) : 1.0");
    println!("Vitesse lue : {}", data[0].speed);
    
}

#[tokio::main]
async fn main() {
    let _ = test_std430().await;
}