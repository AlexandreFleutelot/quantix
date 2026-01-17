use quantix::helpers;
use wgpu;

pub async fn test_marshalling() {

    // use helpers to define the GPU context
    let ctx = helpers::GpuContext::new().await;

    // Create a storage buffer
    let storage_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Storage Buffer quantix"),
        size: 4096,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // Write to the storage buffer
    let data: Vec<u32> = (0..1024).collect();
    let bm_data = bytemuck::cast_slice(&data);
    ctx.queue.write_buffer(&storage_buffer, 0, &bm_data);

    // Create a staging buffer (for reading the data with cpu)
    let staging_buffer = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer quantix"),
        size: 4096,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    //Creation d'une commande de copy
    let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("quantix command encoder")
    });
    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, 4096);
    ctx.queue.submit([encoder.finish()]);

    // Read from the staging buffer
    let buffer_slice = staging_buffer.slice(..);

    //gestion du callback GPU vs await de rust
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();

    //demande de lecture avec callback, prepare les donnée puis appel le callback
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    //Synchro GPU / CPU pour reccup le callback
    ctx.device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    }).expect("Echec de polling du device");

    // Lecture des données...
    if let Some(Ok(())) = receiver.receive().await {
        let data_view = staging_buffer.slice(..).get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data_view).to_vec();

        drop(data_view); // On lache la vue
        staging_buffer.unmap(); // On libere le buffer

        println!("Result: {:?}", result);
    };
}
#[tokio::main]
async fn main() {
    let _ = test_marshalling().await;
}