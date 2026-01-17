use wgpu;

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_info: wgpu::AdapterInfo,
}

impl GpuContext {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await
            .expect("Echec de la recherche d'un adapter GPU");

        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
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
            .expect("Echec de la creation du device");

        Self { device, queue, adapter_info }
    }

    /// Utilitaire pour lire les données d'un buffer GPU (Marshalling sortant)
    pub async fn read_buffer<T: bytemuck::Pod>(&self, buffer: &wgpu::Buffer, size: u64) -> Vec<T> {
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_buffer"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging_buffer, 0, size);
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("Echec du polling");

        if let Some(Ok(())) = receiver.receive().await {
            let data_view = staging_buffer.slice(..).get_mapped_range();
            let result = bytemuck::cast_slice(&data_view).to_vec();
            drop(data_view);
            staging_buffer.unmap();
            result
        } else {
            panic!("Erreur de lecture buffer");
        }
    }
}