use wgpu;

/*
NOTES:
instance: Initialiser la couche logicielle de base (recherche API).
adapter: représentation de réelle de la carte graphique
device: connexion logique ouverte sur l'Adapter (gestionnaire).
queue: la file d'execution
 */
pub async fn get_gpu_context(){
    // Create a GPU instance
    let instance = wgpu::Instance::default();

    // Create an adapter on the instance to connect to the real physical GPU
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {

            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .expect("Echec lors de la création de l'Adapter");

    let adapter_info = adapter.get_info();
    println!("Adapter created with: {:?}", adapter_info);

    // Create the device
    let (_device, _queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("quantix GPU"),
        required_features:wgpu::Features::TIMESTAMP_QUERY, // Pour mesurer les perf
        required_limits: wgpu::Limits {
            max_storage_buffer_binding_size: 512 * 1024 * 1024, // 512 Mo
            ..Default::default()
        },
        ..Default::default()
    }).await
        .expect("Echec de la creation du device ");
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let _ = get_gpu_context().await;
}