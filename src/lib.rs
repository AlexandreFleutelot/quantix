use pollster;
use wgpu;

use pyo3::prelude::*;

#[pyfunction]
fn check_gpu() -> PyResult<()> {
    pollster::block_on(run_gpu_setup());
    Ok(())
}

#[pymodule]
fn quantix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(check_gpu, m)?)?;
    Ok(())
}

// Retourne un tuple contenant le Device et la Queue
pub async fn create_device_and_queue(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
    adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("quantixGPU"),
            required_features: wgpu::Features::TIMESTAMP_QUERY, // TODO: Pour mesurer les perf, a retirer plus tard
            required_limits: wgpu::Limits {
                max_storage_buffer_binding_size: 512 * 1024 * 1024,
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .expect("Echec de la creation du device")
}

pub async fn run_gpu_setup() {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await
        .expect("Echec de la recherche d'un adapter GPU");

    let info = adapter.get_info();
    println!(
        "GPU selectionne : {:?} sur backend {:?}",
        info.name, info.backend
    );

    let (device, queue) = create_device_and_queue(&adapter).await;

    // test storage buffer
    let input_data: Vec<u32> = (1..1024).collect();
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quantix_storage_buffer"),
        size: 4096, //1024 * 4 = 4096
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    //test write buffer
    queue.write_buffer(&storage_buffer, 0, bytemuck::cast_slice(&input_data));
    println!(
        "Buffer written ({} bytes)",
        input_data.len() * std::mem::size_of::<u32>()
    );

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("quantix_staging_buffer"),
        size: 4096,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Encoder de copie"),
    });

    encoder.copy_buffer_to_buffer(&storage_buffer, 0, &staging_buffer, 0, 1024 * 4);

    queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    }).expect("Echec du polling du device");

    if let Some(Ok(())) = receiver.receive().await {
        // Lecture des données...
        let data_view = staging_buffer.slice(..).get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data_view).to_vec();

        drop(data_view); // On lache la vue
        staging_buffer.unmap(); // On libere le buffer

        println!("Result: {:?}", result);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_setup() {
        pollster::block_on(run_gpu_setup());
    }
}
